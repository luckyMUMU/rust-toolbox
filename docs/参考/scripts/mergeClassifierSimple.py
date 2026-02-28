import os
import shutil
import argparse
import logging
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import Dict, List, Tuple, Optional
from threading import Lock
from collections import OrderedDict
import time

# 设置 stdout 编码以支持中文输出
if sys.platform == 'win32':
    import io
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')

# 配置日志
LOG_FORMAT = "%(asctime)s - %(levelname)s - %(threadName)s - %(message)s"
logging.basicConfig(level=logging.INFO, format=LOG_FORMAT, encoding='utf-8')

class SimpleFileMover:
    """简化版文件移动器，支持多线程"""
    
    def __init__(self, max_workers: int = 12):
        self.executor = ThreadPoolExecutor(max_workers=max_workers, thread_name_prefix="FileMover")
        self._success_count = 0
        self._failure_count = 0
        self._lock = Lock()
        self._rename_lock = Lock()
    
    @property
    def success_count(self) -> int:
        return self._success_count
    
    @property
    def failure_count(self) -> int:
        return self._failure_count
    
    def _increment_success(self):
        with self._lock:
            self._success_count += 1
    
    def _increment_failure(self):
        with self._lock:
            self._failure_count += 1
    
    def move_file(self, src: str, dst: str) -> bool:
        """移动文件，处理重复文件重命名"""
        try:
            with self._rename_lock:
                if os.path.exists(dst):
                    dst = self._rename_duplicate_file(dst)
                    logging.debug(f"文件已重命名: {dst}")
                shutil.move(src, dst)
            logging.debug(f"移动文件 '{src}' 到 '{dst}'")
            self._increment_success()
            return True
        except Exception:
            logging.exception(f"移动文件失败：'{src}' 到 '{dst}'")
            self._increment_failure()
            return False
    
    def move_folder(self, src: str, dst: str) -> bool:
        """移动文件夹"""
        try:
            if os.path.exists(dst):
                logging.error(f"移动文件夹失败：目标文件夹 '{dst}' 已存在")
                self._increment_failure()
                return False
            
            shutil.move(src, dst)
            logging.debug(f"移动文件夹 '{src}' 到 '{dst}'")
            self._increment_success()
            return True
        except PermissionError as e:
            logging.error(f"移动文件夹失败（权限不足）：'{src}' 到 '{dst}' - {e}")
            self._increment_failure()
            return False
        except OSError as e:
            logging.error(f"移动文件夹失败（系统错误）：'{src}' 到 '{dst}' - 错误码: {e.errno}, 信息: {e.strerror}")
            self._increment_failure()
            return False
        except Exception:
            logging.exception(f"移动文件夹失败（未知错误）：'{src}' 到 '{dst}'")
            self._increment_failure()
            return False
    
    def move_file_async(self, src: str, dst: str):
        """异步移动文件"""
        return self.executor.submit(self.move_file, src, dst)
    
    def move_folder_async(self, src: str, dst: str):
        """异步移动文件夹"""
        return self.executor.submit(self.move_folder, src, dst)
    
    def _rename_duplicate_file(self, filepath: str) -> str:
        """重命名重复文件，添加'move'后缀"""
        directory, filename = os.path.split(filepath)
        name, ext = os.path.splitext(filename)
        
        # 添加" move"后缀
        new_filename = f"{name} move{ext}"
        new_filepath = os.path.join(directory, new_filename)
        
        # 如果重命名后仍然冲突，添加数字后缀
        counter = 1
        while os.path.exists(new_filepath):
            new_filename = f"{name} move ({counter}){ext}"
            new_filepath = os.path.join(directory, new_filename)
            counter += 1
            
        return new_filepath
    
    def close(self):
        """关闭线程池"""
        self.executor.shutdown(wait=True)
        logging.info(f"文件移动完成：成功 {self.success_count} 个，失败 {self.failure_count} 个")

class LRUCache:
    """线程安全的LRU缓存"""
    
    def __init__(self, max_size: int = 1000):
        self._cache: OrderedDict = OrderedDict()
        self._max_size = max_size
        self._lock = Lock()
    
    def get(self, key: str, default=None):
        with self._lock:
            if key in self._cache:
                self._cache.move_to_end(key)
                return self._cache[key]
            return default
    
    def __contains__(self, key: str) -> bool:
        with self._lock:
            return key in self._cache
    
    def set(self, key: str, value):
        with self._lock:
            if key in self._cache:
                self._cache.move_to_end(key)
            self._cache[key] = value
            if len(self._cache) > self._max_size:
                self._cache.popitem(last=False)

class SimpleFolderMerger:
    """简化版文件夹合并器，支持多线程处理"""
    
    MAX_RECURSION_DEPTH = 10
    MAX_PARALLEL_ITEMS = 200
    MAX_CACHE_SIZE = 1000
    
    def __init__(self, max_workers: int = 4):
        self.file_mover = SimpleFileMover(max_workers)
        self.executor = ThreadPoolExecutor(max_workers=max_workers, thread_name_prefix="FolderMerger")
        self._size_cache = LRUCache(self.MAX_CACHE_SIZE)
        self._disk_space_cache = LRUCache(self.MAX_CACHE_SIZE)
    
    def get_folder_size(self, folder_path: str) -> int:
        """计算文件夹的大小（字节），同步实现避免线程池死锁"""
        folder_path = os.path.abspath(folder_path)
        cached = self._size_cache.get(folder_path)
        if cached is not None:
            return cached
        
        total_size = 0
        for dirpath, dirnames, filenames in os.walk(folder_path):
            total_size += self._calculate_dir_size(dirpath, filenames)
        
        self._size_cache.set(folder_path, total_size)
        return total_size
    
    def _calculate_dir_size(self, dirpath: str, filenames: List[str]) -> int:
        """计算单个目录的大小"""
        size = 0
        for f in filenames:
            fp = os.path.join(dirpath, f)
            if not os.path.islink(fp):
                try:
                    size += os.path.getsize(fp)
                except (OSError, FileNotFoundError) as e:
                    # 文件可能在计算过程中被删除或无法访问
                    logging.debug(f"无法获取文件大小 '{fp}'：{e}")
        return size
    
    def get_disk_free_space(self, folder_path: str) -> int:
        """获取路径所在驱动器的可用空间"""
        try:
            abs_path = os.path.abspath(folder_path)
            drive = os.path.splitdrive(abs_path)[0]
            
            if not drive:
                drive = abs_path
            
            cached = self._disk_space_cache.get(drive)
            if cached is not None:
                return cached
            
            total, used, free = shutil.disk_usage(drive)
            self._disk_space_cache.set(drive, free)
            return free
        except PermissionError as e:
            logging.error(f"获取磁盘空间失败（权限不足） '{folder_path}': {e}")
            return -1
        except OSError as e:
            logging.error(f"获取磁盘空间失败（系统错误） '{folder_path}': 错误码: {e.errno}, 信息: {e.strerror}")
            return -1
        except Exception:
            logging.exception(f"获取磁盘空间失败（未知错误） '{folder_path}'")
            return -1

    def merge_folders(self, source_folder: str, destination_folder: str, depth: int = 0) -> bool:
        """将源文件夹的内容合并到目标文件夹，并检查磁盘空间
        
        Args:
            source_folder: 源文件夹路径
            destination_folder: 目标文件夹路径
            depth: 当前递归深度，用于限制递归层数
            
        Returns:
            合并是否成功
        """
        logging.info(f"开始合并文件夹：从 '{source_folder}' 到 '{destination_folder}' (深度: {depth})")
        
        if depth > self.MAX_RECURSION_DEPTH:
            logging.warning(f"  递归深度超过限制 ({self.MAX_RECURSION_DEPTH})，跳过该子文件夹：{source_folder}")
            return False
        
        os.makedirs(destination_folder, exist_ok=True)
        
        try:
            items = os.listdir(source_folder)
            if not items:
                logging.info(f"源文件夹 '{source_folder}' 为空，无需合并")
                try:
                    os.rmdir(source_folder)
                except OSError:
                    pass
                return True
            
            total_items = len(items)
            processed_items = 0
            has_error = False
            
            batch_size = min(self.MAX_PARALLEL_ITEMS, total_items)
            
            for i in range(0, total_items, batch_size):
                batch = items[i:i+batch_size]
                file_futures = []
                
                for item in batch:
                    s_item = os.path.join(source_folder, item)
                    d_item = os.path.join(destination_folder, item)
                    
                    if os.path.isdir(s_item) and not item.startswith('.'):
                        # 使用同步递归避免线程池死锁
                        result = self.merge_folders(s_item, d_item, depth + 1)
                        if not result:
                            has_error = True
                        processed_items += 1
                    elif os.path.isfile(s_item):
                        future = self.file_mover.move_file_async(s_item, d_item)
                        file_futures.append((future, s_item, d_item))
                    elif os.path.islink(s_item):
                        logging.warning(f"跳过符号链接：{s_item}")
                        processed_items += 1
                    else:
                        logging.warning(f"跳过未知类型项目：{s_item}")
                        processed_items += 1
                
                # 等待文件移动任务完成
                for future, src, dst in file_futures:
                    try:
                        result = future.result()
                        if not result:
                            has_error = True
                        processed_items += 1
                    except Exception:
                        logging.exception(f"移动文件失败：'{src}' 到 '{dst}'")
                        has_error = True
                        processed_items += 1
                
                if total_items > 100:
                    progress = (processed_items / total_items) * 100
                    logging.info(f"  合并进度：{processed_items}/{total_items} ({progress:.1f}%)")
            
            if not has_error:
                try:
                    shutil.rmtree(source_folder)
                    logging.info(f"成功删除源文件夹 '{source_folder}'")
                except Exception:
                    logging.exception(f"删除源文件夹失败 '{source_folder}'")
            else:
                logging.warning(f"由于合并过程中出现错误，保留源文件夹 '{source_folder}'")
            
            return not has_error
            
        except Exception:
            logging.exception(f"合并文件夹失败：'{source_folder}' 到 '{destination_folder}'")
            return False
    
    def merge_folders_async(self, source_folder: str, destination_folder: str, timeout: float = None):
        """异步合并文件夹
        
        Args:
            source_folder: 源文件夹路径
            destination_folder: 目标文件夹路径
            timeout: 超时时间（秒），None表示无超时
        """
        return self.executor.submit(self.merge_folders, source_folder, destination_folder)
    
    def close(self, timeout: float = 300):
        """关闭资源
        
        Args:
            timeout: 等待线程池关闭的超时时间（秒），默认5分钟
        """
        self.file_mover.close()
        self.executor.shutdown(wait=True, cancel_futures=False)
        logging.info("所有线程池已关闭")

class FolderMergeResult:
    """文件夹合并结果"""
    def __init__(self, subfolder_name: str, source: str, destination: str, success: bool, message: str):
        self.subfolder_name = subfolder_name
        self.source = source
        self.destination = destination
        self.success = success
        self.message = message

class EnhancedFolderMerger(SimpleFolderMerger):
    """增强版文件夹合并器，支持多文件夹合并"""
    
    def __init__(self, max_workers: int = 4):
        super().__init__(max_workers)
        self.merge_results: List[FolderMergeResult] = []
    
    def merge_common_subfolders(self, folders: List[str]) -> List[FolderMergeResult]:
        """合并多个根文件夹下的同名子文件夹"""
        
        try:
            # 重置合并结果列表
            self.merge_results = []
            
            # 1. 扫描所有根文件夹，建立映射 {subfolder_name: [full_path1, full_path2, ...]}
            subfolder_map = {} # str -> List[str]
            
            for root_folder in folders:
                if not os.path.exists(root_folder):
                    logging.warning(f"文件夹不存在，跳过: {root_folder}")
                    continue
                    
                try:
                    for entry in os.scandir(root_folder):
                        if entry.is_dir() and not entry.name.startswith('.'):
                            if entry.name not in subfolder_map:
                                subfolder_map[entry.name] = []
                            subfolder_map[entry.name].append(entry.path)
                except Exception as e:
                    logging.error(f"扫描文件夹失败 '{root_folder}': {e}")

            # 2. 找出出现在多个根文件夹中的子文件夹
            common_subfolders = {name: paths for name, paths in subfolder_map.items() if len(paths) > 1}

            if not common_subfolders:
                logging.info("没有找到同名子文件夹，无需合并。")
                return []

            # 3. 并行处理所有同名子文件夹
            futures = {}
            for subfolder_name, paths in common_subfolders.items():
                
                # 计算每个路径的大小
                path_sizes = []
                for p in paths:
                    size = self.get_folder_size(p)
                    path_sizes.append((p, size))
                
                logging.info(f"发现同名子文件夹 '{subfolder_name}' 在 {len(paths)} 个位置:")
                for p, s in path_sizes:
                    logging.info(f"  '{p}' 大小: {s:,} 字节")

                # 按大小降序排序，最大的作为首选目标
                sorted_candidates = sorted(path_sizes, key=lambda x: x[1], reverse=True)
                
                merge_plan_success = False
                
                for i, (candidate_target, candidate_size) in enumerate(sorted_candidates):
                    # 假设这个是 Target，其他的都是 Sources
                    current_sources = [entry for j, entry in enumerate(sorted_candidates) if i != j]
                    
                    # 计算总共需要的空间
                    total_required = sum(size for _, size in current_sources)
                    free_space = self.get_disk_free_space(candidate_target)
                    
                    if free_space < 0:
                        logging.warning(f"  目标 '{candidate_target}' 无法获取磁盘空间信息，尝试下一个候选...")
                        continue
                    
                    if free_space >= total_required:
                        logging.info(f"  选定目标: '{candidate_target}' (可用空间: {free_space:,} 字节, 需合并: {total_required:,} 字节)")
                        
                        # 执行合并：将所有 sources 移动到 candidate_target
                        for src_path, src_size in current_sources:
                            future = self.merge_folders_async(src_path, candidate_target)
                            futures[future] = (subfolder_name, src_path, candidate_target, "空间充足")
                        
                        merge_plan_success = True
                        break
                    else:
                        logging.warning(f"  目标 '{candidate_target}' 空间不足 (可用: {free_space:,}, 需: {total_required:,})，尝试下一个候选...")
                
                if not merge_plan_success:
                     # 没有任何一个文件夹能装下其他的
                    logging.error(f"  无法合并 '{subfolder_name}': 所有候选目标的磁盘空间都不足")
                    self.merge_results.append(FolderMergeResult(
                        subfolder_name, "Multiple Sources", "No Valid Target", False, 
                        "所有候选目标的磁盘空间都不足"
                    ))

            # 处理合并结果（使用 as_completed 防止假死）
            for future in as_completed(futures):
                try:
                    # 获取该 future 对应的上下文信息
                    subfolder_name, source, destination, message = futures[future]
                    
                    result = future.result()
                    if result:
                        logging.info(f"  成功合并 '{source}' -> '{destination}'")
                        self.merge_results.append(FolderMergeResult(
                            subfolder_name, source, destination, True, "合并成功"
                        ))
                    else:
                        logging.error(f"  合并 '{source}' 失败")
                        self.merge_results.append(FolderMergeResult(
                            subfolder_name, source, destination, False, "合并失败"
                        ))
                except Exception:
                    context = futures.get(future)
                    if context:
                        subfolder_name, source, destination, _ = context
                        logging.exception(f"  合并 '{subfolder_name}' 异常")
                        self.merge_results.append(FolderMergeResult(
                            subfolder_name, source, destination, False, "合并异常"
                        ))
                    else:
                        logging.exception(f"  未知任务异常")

            return self.merge_results
            
        except Exception:
            logging.exception("合并文件夹失败")
            return self.merge_results

def main():
    parser = argparse.ArgumentParser(description="比较并合并多个文件夹下的同名子文件夹（增强版）。")
    parser.add_argument("folders", nargs='+', help="要合并的根文件夹路径列表")
    parser.add_argument("--workers", type=int, default=4, help="并发工作线程数（默认：4）")
    parser.add_argument("--debug", action="store_true", help="启用调试日志")
    args = parser.parse_args()

    folders = [os.path.abspath(f) for f in args.folders]
    max_workers = args.workers

    # 验证文件夹
    valid_folders = []
    for f in folders:
        if os.path.isdir(f):
            valid_folders.append(f)
        else:
            logging.error(f"错误：文件夹 '{f}' 不存在或不是一个有效的目录，跳过。")
    
    if len(valid_folders) < 2:
        logging.error("错误：至少需要提供两个有效的文件夹路径进行合并。")
        return

    # 设置日志级别
    if args.debug:
        logging.getLogger().setLevel(logging.DEBUG)

    start_time = time.time()
    
    # 使用增强版文件夹合并器
    merger = EnhancedFolderMerger(max_workers)
    
    try:
        results = merger.merge_common_subfolders(valid_folders)
        
        # 打印合并结果摘要
        print("\n" + "="*60)
        print("文件夹合并结果摘要")
        print("="*60)
        
        if not results:
            print("没有执行任何合并操作")
        else:
            success_count = sum(1 for r in results if r.success)
            failure_count = len(results) - success_count
            
            print(f"总计处理: {len(results)} 个合并任务")
            print(f"成功合并: {success_count} 个")
            print(f"合并失败: {failure_count} 个")
            
            if failure_count > 0:
                print("\n失败详情:")
                for result in results:
                    if not result.success:
                        print(f"  - {result.subfolder_name}: {result.message} (Src: {result.source}, Dst: {result.destination})")
        
    finally:
        merger.close()
    
    end_time = time.time()
    logging.info(f"合并操作完成，耗时: {end_time - start_time:.2f} 秒")

if __name__ == "__main__":
    main()
