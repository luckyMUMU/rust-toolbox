import os
import shutil
import argparse
import logging
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import Dict, List, Tuple, Optional
import time

# 配置日志
LOG_FORMAT = "%(asctime)s - %(levelname)s - %(threadName)s - %(message)s"
logging.basicConfig(level=logging.INFO, format=LOG_FORMAT, encoding='utf-8')

class SimpleFileMover:
    """简化版文件移动器，支持多线程"""
    
    def __init__(self, max_workers: int = 12):
        self.executor = ThreadPoolExecutor(max_workers=max_workers, thread_name_prefix="FileMover")
        self.success_count = 0
        self.failure_count = 0
    
    def move_file(self, src: str, dst: str) -> bool:
        """移动文件，处理重复文件重命名"""
        try:
            # 检查目标文件是否存在，如果存在则重命名
            if os.path.exists(dst):
                dst = self._rename_duplicate_file(dst)
                logging.debug(f"文件已重命名: {dst}")
            
            # 移动文件
            shutil.move(src, dst)
            logging.debug(f"移动文件 '{src}' 到 '{dst}'")
            self.success_count += 1
            return True
        except Exception as e:
            logging.error(f"移动文件失败：'{src}' 到 '{dst}'。错误：{e}")
            self.failure_count += 1
            return False
    
    def move_folder(self, src: str, dst: str) -> bool:
        """移动文件夹"""
        try:
            # 检查目标文件夹是否存在
            if os.path.exists(dst):
                logging.error(f"移动文件夹失败：目标文件夹 '{dst}' 已存在")
                self.failure_count += 1
                return False
            
            shutil.move(src, dst)
            logging.debug(f"移动文件夹 '{src}' 到 '{dst}'")
            self.success_count += 1
            return True
        except Exception as e:
            logging.error(f"移动文件夹失败：'{src}' 到 '{dst}'。错误：{e}")
            self.failure_count += 1
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

class SimpleFolderMerger:
    """简化版文件夹合并器，支持多线程处理"""
    
    def __init__(self, max_workers: int = 4):
        self.file_mover = SimpleFileMover(max_workers)
        self.executor = ThreadPoolExecutor(max_workers=max_workers, thread_name_prefix="FolderMerger")
    
    def get_folder_size(self, folder_path: str) -> int:
        """计算文件夹的大小（字节），使用多线程加速"""
        total_size = 0
        
        # 使用类中已有的线程池并行计算子文件夹大小
        futures = []
        for dirpath, dirnames, filenames in os.walk(folder_path):
            future = self.executor.submit(self._calculate_dir_size, dirpath, filenames)
            futures.append(future)
        
        for future in as_completed(futures):
            total_size += future.result()
                
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
        drive = os.path.splitdrive(os.path.abspath(folder_path))[0]
        total, used, free = shutil.disk_usage(drive)
        return free
    
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
        
        # 限制递归深度，防止过深的文件夹结构导致性能问题
        MAX_RECURSION_DEPTH = 10
        if depth > MAX_RECURSION_DEPTH:
            logging.warning(f"  递归深度超过限制 ({MAX_RECURSION_DEPTH})，跳过该子文件夹：{source_folder}")
            return False
        
        # 确保目标文件夹存在
        os.makedirs(destination_folder, exist_ok=True)
        
        try:
            # 获取源文件夹中的所有项目
            items = os.listdir(source_folder)
            if not items:
                logging.info(f"源文件夹 '{source_folder}' 为空，无需合并")
                return True
            
            # 统计项目数量，用于进度报告
            total_items = len(items)
            processed_items = 0
            
            # 优化：对于大型文件夹，使用批量处理而非完全并行
            MAX_PARALLEL_ITEMS = 200
            batch_size = min(MAX_PARALLEL_ITEMS, total_items)
            
            # 分批处理项目
            for i in range(0, total_items, batch_size):
                batch = items[i:i+batch_size]
                futures = []
                
                for item in batch:
                    s_item = os.path.join(source_folder, item)
                    d_item = os.path.join(destination_folder, item)
                    
                    if os.path.isdir(s_item) and not item.startswith('.'):
                        # 对于子文件夹，递归合并
                        future = self.executor.submit(self.merge_folders, s_item, d_item, depth + 1)
                        futures.append(future)
                    elif os.path.isfile(s_item):
                        # 对于文件，异步移动
                        future = self.file_mover.move_file_async(s_item, d_item)
                        futures.append(future)
                    elif os.path.islink(s_item):
                        # 处理符号链接
                        logging.warning(f"跳过符号链接：{s_item}")
                        processed_items += 1
                    else:
                        logging.warning(f"跳过未知类型项目：{s_item}")
                        processed_items += 1
                
                # 等待当前批次完成
                for future in as_completed(futures):
                    try:
                        result = future.result()
                        if result is False:
                            return False
                        processed_items += 1
                    except Exception as e:
                        logging.error(f"合并项目失败：{e}")
                        return False
                
                # 输出进度报告
                if total_items > 100:
                    progress = (processed_items / total_items) * 100
                    logging.info(f"  合并进度：{processed_items}/{total_items} ({progress:.1f}%)")
            
            # 只有当所有项目都成功移动后，才删除源文件夹
            shutil.rmtree(source_folder)
            logging.info(f"成功删除源文件夹 '{source_folder}'")
            return True
            
        except Exception as e:
            logging.error(f"合并文件夹失败：'{source_folder}' 到 '{destination_folder}'。错误：{e}")
            return False
    
    def merge_folders_async(self, source_folder: str, destination_folder: str):
        """异步合并文件夹"""
        return self.executor.submit(self.merge_folders, source_folder, destination_folder)
    
    def close(self):
        """关闭资源"""
        self.file_mover.close()
        self.executor.shutdown(wait=True)
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
    """增强版文件夹合并器，添加了更多功能"""
    
    def __init__(self, max_workers: int = 4):
        super().__init__(max_workers)
        self.merge_results: List[FolderMergeResult] = []
    
    def merge_common_subfolders(self, folder1: str, folder2: str) -> List[FolderMergeResult]:
        """合并两个根文件夹下的同名子文件夹"""
        
        try:
            # 重置合并结果列表
            self.merge_results = []
            
            # 获取子文件夹列表
            subfolders1 = {f.name: os.path.join(folder1, f.name) for f in os.scandir(folder1) if f.is_dir() and not f.name.startswith('.')}
            subfolders2 = {f.name: os.path.join(folder2, f.name) for f in os.scandir(folder2) if f.is_dir() and not f.name.startswith('.')}

            common_subfolders = set(subfolders1.keys()).intersection(set(subfolders2.keys()))

            if not common_subfolders:
                logging.info("没有找到同名子文件夹，无需合并。")
                return []

            # 并行处理所有同名子文件夹
            futures = {}
            for subfolder_name in common_subfolders:
                path1 = subfolders1[subfolder_name]
                path2 = subfolders2[subfolder_name]

                size1 = self.get_folder_size(path1)
                size2 = self.get_folder_size(path2)

                logging.info(f"发现同名子文件夹 '{subfolder_name}':")
                logging.info(f"  '{path1}' 大小: {size1:,} 字节")
                logging.info(f"  '{path2}' 大小: {size2:,} 字节")

                # 确定合并方向
                if size1 < size2:
                    smaller_folder, larger_folder = path1, path2
                    logging.info(f"  '{smaller_folder}' 较小，将合并到 '{larger_folder}'")
                else:
                    smaller_folder, larger_folder = path2, path1
                    logging.info(f"  '{smaller_folder}' 较小，将合并到 '{larger_folder}'")

                # 检查文件夹大小，只有当两边都大于2GB时才跳过合并
                MAX_FOLDER_SIZE = 2 * 1024 * 1024 * 1024  # 2GB 大小限制
                
                if size1 > MAX_FOLDER_SIZE and size2 > MAX_FOLDER_SIZE:
                    logging.warning(f"  文件夹 '{subfolder_name}' 两边都超过2GB限制，跳过合并")
                    self.merge_results.append(FolderMergeResult(
                        subfolder_name, smaller_folder, larger_folder, False, 
                        f"文件夹过大: 两边都超过2GB限制（{size1:,} 字节 和 {size2:,} 字节）"
                    ))
                    continue
                
                # 检查目标驱动器的可用空间
                free_space = self.get_disk_free_space(larger_folder)
                required_space = size1 if size1 < size2 else size2
                
                if free_space >= required_space:
                    # 异步合并文件夹
                    future = self.merge_folders_async(smaller_folder, larger_folder)
                    futures[future] = (subfolder_name, smaller_folder, larger_folder, "空间充足")
                else:
                    # 正向合并空间不足，尝试反向合并
                    logging.warning(f"  正向合并驱动器 '{os.path.splitdrive(larger_folder)[0]}' 空间不足 (需要: {required_space:,} 字节, 可用: {free_space:,} 字节)")
                    logging.info(f"  尝试反向合并：将 '{larger_folder}' 合并到 '{smaller_folder}'")
                    
                    # 检查反向合并的目标驱动器可用空间
                    reverse_free_space = self.get_disk_free_space(smaller_folder)
                    reverse_required_space = size2 if size1 < size2 else size1
                    
                    if reverse_free_space >= reverse_required_space:
                        # 反向合并空间充足，执行反向合并
                        future = self.merge_folders_async(larger_folder, smaller_folder)
                        futures[future] = (subfolder_name, larger_folder, smaller_folder, "反向合并空间充足")
                    else:
                        # 双向合并都空间不足，记录失败
                        logging.error(f"  反向合并驱动器 '{os.path.splitdrive(smaller_folder)[0]}' 也空间不足 (需要: {reverse_required_space:,} 字节, 可用: {reverse_free_space:,} 字节)")
                        self.merge_results.append(FolderMergeResult(
                            subfolder_name, smaller_folder, larger_folder, False, 
                            f"双向合并都空间不足: 正向需要 {required_space:,} 字节, 反向需要 {reverse_required_space:,} 字节"
                        ))

            # 处理合并结果
            for future, (subfolder_name, source, destination, message) in futures.items():
                try:
                    result = future.result()
                    if result:
                        logging.info(f"  成功合并 '{subfolder_name}'")
                        self.merge_results.append(FolderMergeResult(
                            subfolder_name, source, destination, True, "合并成功"
                        ))
                    else:
                        logging.error(f"  合并 '{subfolder_name}' 失败")
                        self.merge_results.append(FolderMergeResult(
                            subfolder_name, source, destination, False, "合并失败"
                        ))
                except Exception as e:
                    logging.error(f"  合并 '{subfolder_name}' 异常: {e}")
                    self.merge_results.append(FolderMergeResult(
                        subfolder_name, source, destination, False, f"合并异常: {e}"
                    ))

            return self.merge_results
                
        except Exception as e:
            logging.error(f"合并文件夹失败：{e}")
            return self.merge_results

def main():
    parser = argparse.ArgumentParser(description="比较并合并两个文件夹下的同名子文件夹（增强版）。")
    parser.add_argument("folder1", help="第一个根文件夹的路径")
    parser.add_argument("folder2", help="第二个根文件夹的路径")
    parser.add_argument("--workers", type=int, default=4, help="并发工作线程数（默认：4）")
    parser.add_argument("--debug", action="store_true", help="启用调试日志")
    args = parser.parse_args()

    folder1_path = os.path.abspath(args.folder1)
    folder2_path = os.path.abspath(args.folder2)
    max_workers = args.workers

    if not os.path.isdir(folder1_path):
        logging.error(f"错误：文件夹 '{folder1_path}' 不存在或不是一个有效的目录。")
        return
    if not os.path.isdir(folder2_path):
        logging.error(f"错误：文件夹 '{folder2_path}' 不存在或不是一个有效的目录。")
        return

    # 设置日志级别
    if args.debug:
        logging.getLogger().setLevel(logging.DEBUG)

    start_time = time.time()
    
    # 使用增强版文件夹合并器
    merger = EnhancedFolderMerger(max_workers)
    
    try:
        results = merger.merge_common_subfolders(folder1_path, folder2_path)
        
        # 打印合并结果摘要
        print("\n" + "="*60)
        print("文件夹合并结果摘要")
        print("="*60)
        
        if not results:
            print("没有执行任何合并操作")
        else:
            success_count = sum(1 for r in results if r.success)
            failure_count = len(results) - success_count
            
            print(f"总计处理: {len(results)} 个同名子文件夹")
            print(f"成功合并: {success_count} 个")
            print(f"合并失败: {failure_count} 个")
            
            if failure_count > 0:
                print("\n失败详情:")
                for result in results:
                    if not result.success:
                        print(f"  - {result.subfolder_name}: {result.message}")
        
    finally:
        merger.close()
    
    end_time = time.time()
    logging.info(f"合并操作完成，耗时: {end_time - start_time:.2f} 秒")

if __name__ == "__main__":
    main()