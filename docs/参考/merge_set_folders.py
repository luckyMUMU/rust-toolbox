import os
import shutil
import argparse
import logging
import re
import sys
from datetime import datetime
from pathlib import Path


def setup_logging(log_file='merge_set_folders.log'):
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(levelname)s - %(message)s',
        handlers=[
            logging.FileHandler(log_file, encoding='utf-8'),
            logging.StreamHandler()
        ]
    )
    return logging.getLogger(__name__)


def setup_stdout_encoding():
    """设置 stdout 编码以支持中文输出"""
    if sys.platform == 'win32':
        import io
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')


def number_to_letter(number):
    """将数字转换为小写英文字母（1->a, 2->b, ..., 26->z, 27->aa, ...）"""
    result = []
    while number > 0:
        number -= 1
        result.append(chr(ord('a') + (number % 26)))
        number //= 26
    return ''.join(reversed(result))


def parse_set_folder(folder_name):
    """解析文件夹名称，返回(基础名称, 数字)或None"""
    # 支持的格式: 
    # - Name Set.01, Name Set 01
    # - Name part.01, Name part01, Name part 01
    # - Name vol.01, Name vol01
    pattern = r'^(.+?)[\s._-]+(?:set|part|vol)[\s._-]*(\d+)$'
    match = re.match(pattern, folder_name, re.IGNORECASE)
    if match:
        base_name = match.group(1).strip()
        number = int(match.group(2))
        return base_name, number
    return None


def scan_set_folders(target_directory):
    """扫描目标目录，识别所有Set文件夹并按基础名称分组"""
    set_folders = {}
    
    for root, dirs, files in os.walk(target_directory):
        for dir_name in dirs:
            parsed = parse_set_folder(dir_name)
            if parsed:
                base_name, number = parsed
                full_path = os.path.join(root, dir_name)
                if base_name not in set_folders:
                    set_folders[base_name] = []
                set_folders[base_name].append({
                    'name': dir_name,
                    'path': full_path,
                    'number': number
                })
    
    return set_folders


def merge_folder_group(base_name, folders, logger, dry_run=False):
    """合并同一基础名称的文件夹组"""
    if len(folders) < 2:
        logger.info(f"基础名称 '{base_name}' 只有一个文件夹，无需合并")
        return True
    
    folders.sort(key=lambda x: x['number'])
    target_folder = folders[0]
    source_folders = folders[1:]
    
    logger.info(f"开始合并基础名称 '{base_name}' 的文件夹组")
    logger.info(f"目标文件夹: {target_folder['name']} (Set.{target_folder['number']})")
    
    success = True
    
    for source_folder in source_folders:
        logger.info(f"处理来源文件夹: {source_folder['name']} (Set.{source_folder['number']})")
        
        if not merge_folders(target_folder['path'], source_folder['path'], 
                           source_folder['number'], logger, dry_run):
            success = False
            continue
        
        if not dry_run:
            try:
                os.rmdir(source_folder['path'])
                logger.info(f"已删除空文件夹: {source_folder['path']}")
            except Exception as e:
                logger.error(f"删除文件夹失败 {source_folder['path']}: {e}")
                success = False
    
    return success


def merge_folders(target_path, source_path, source_number, logger, dry_run=False):
    """将source_path中的内容合并到target_path"""
    prefix = number_to_letter(source_number) + '_'
    
    try:
        for item in os.listdir(source_path):
            source_item = os.path.join(source_path, item)
            target_item = os.path.join(target_path, item)
            
            if os.path.isdir(source_item):
                if os.path.exists(target_item):
                    if not merge_folders(target_item, source_item, source_number, logger, dry_run):
                        return False
                else:
                    if dry_run:
                        logger.info(f"[DRY-RUN] 将移动文件夹: {source_item} -> {target_item}")
                    else:
                        shutil.move(source_item, target_item)
                        logger.info(f"已移动文件夹: {source_item} -> {target_item}")
            else:
                if os.path.exists(target_item):
                    new_name = prefix + item
                    target_item = os.path.join(target_path, new_name)
                    logger.info(f"文件冲突，添加前缀 '{prefix}': {item} -> {new_name}")
                
                if dry_run:
                    logger.info(f"[DRY-RUN] 将移动文件: {source_item} -> {target_item}")
                else:
                    shutil.move(source_item, target_item)
                    logger.info(f"已移动文件: {source_item} -> {target_item}")
        
        return True
    except Exception as e:
        logger.error(f"合并文件夹时出错 {source_path}: {e}")
        return False


def merge_set_folders(target_directory, dry_run=False, preview_only=False):
    """主函数：扫描并合并所有Set文件夹"""
    logger = setup_logging()
    
    logger.info("=" * 60)
    logger.info(f"系列文件夹(Set/Part/Vol)合并工具启动")
    logger.info(f"目标目录: {target_directory}")
    logger.info(f"模式: {'预览模式 (DRY-RUN)' if dry_run else '执行模式'}")
    logger.info("=" * 60)
    
    if not os.path.isdir(target_directory):
        logger.error(f"目标目录不存在: {target_directory}")
        return False
    
    set_folders = scan_set_folders(target_directory)
    
    if not set_folders:
        logger.info("未找到任何符合格式(Set/Part/Vol)的文件夹")
        return True
    
    logger.info(f"找到 {len(set_folders)} 个基础名称组")
    for base_name, folders in set_folders.items():
        folder_names = [f['name'] for f in folders]
        logger.info(f"  - {base_name}: {', '.join(folder_names)}")
    
    logger.info("=" * 60)
    
    total_groups = len(set_folders)
    success_count = 0
    
    for base_name, folders in set_folders.items():
        if merge_folder_group(base_name, folders, logger, dry_run):
            success_count += 1
    
    logger.info("=" * 60)
    logger.info(f"合并完成: 成功 {success_count}/{total_groups} 组")
    logger.info("=" * 60)
    
    return success_count == total_groups


def preview_and_confirm(target_directory):
    """预览合并计划并请求用户确认"""
    logger = setup_logging()
    
    logger.info("=" * 60)
    logger.info(f"系列文件夹(Set/Part/Vol)合并工具 - 预览模式")
    logger.info(f"目标目录: {target_directory}")
    logger.info("=" * 60)
    
    if not os.path.isdir(target_directory):
        logger.error(f"目标目录不存在: {target_directory}")
        return False
    
    set_folders = scan_set_folders(target_directory)
    
    if not set_folders:
        logger.info("未找到任何符合格式(Set/Part/Vol)的文件夹")
        return True
    
    logger.info(f"找到 {len(set_folders)} 个基础名称组")
    logger.info("=" * 60)
    
    merge_groups = []
    for base_name, folders in set_folders.items():
        folders.sort(key=lambda x: x['number'])
        if len(folders) >= 2:
            target_folder = folders[0]
            source_folders = folders[1:]
            merge_groups.append({
                'base_name': base_name,
                'target': target_folder,
                'sources': source_folders
            })
            logger.info(f"\n基础名称: {base_name}")
            logger.info(f"  目标文件夹: {target_folder['name']} (Set.{target_folder['number']})")
            logger.info(f"  来源文件夹: {', '.join([f['name'] + ' (Set.' + str(f['number']) + ')' for f in source_folders])}")
        else:
            logger.info(f"\n基础名称: {base_name}")
            logger.info(f"  只有一个文件夹，无需合并: {folders[0]['name']}")
    
    if not merge_groups:
        logger.info("\n没有需要合并的文件夹组")
        return True
    
    logger.info("=" * 60)
    logger.info(f"总计需要合并 {len(merge_groups)} 个文件夹组")
    logger.info("=" * 60)
    
    print("\n" + "=" * 60)
    print("是否确认执行合并操作？")
    print("输入 'y' 或 'yes' 确认执行，其他任意键取消")
    print("=" * 60)
    
    user_input = input("\n请确认 (y/yes): ").strip().lower()
    
    if user_input in ['y', 'yes']:
        print("\n开始执行合并操作...")
        logger.info("用户确认执行合并操作")
        return merge_set_folders(target_directory, dry_run=False)
    else:
        print("\n操作已取消")
        logger.info("用户取消合并操作")
        return True


def main():
    parser = argparse.ArgumentParser(
        description='合并名称格式为"[基础名称] Set/Part/Vol.XX"的系列文件夹'
    )
    parser.add_argument(
        'target_directory',
        help='目标文件夹路径'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='预览模式，不实际执行移动和删除操作（不请求确认）'
    )
    parser.add_argument(
        '--yes',
        action='store_true',
        help='跳过确认直接执行合并操作'
    )
    
    args = parser.parse_args()
    
    if args.dry_run:
        if merge_set_folders(args.target_directory, dry_run=True):
            print("\n预览操作完成！")
            return 0
        else:
            print("\n预览过程中出现错误，请查看日志获取详细信息。")
            return 1
    elif args.yes:
        if merge_set_folders(args.target_directory, dry_run=False):
            print("\n操作成功完成！")
            return 0
        else:
            print("\n操作过程中出现错误，请查看日志获取详细信息。")
            return 1
    else:
        if preview_and_confirm(args.target_directory):
            return 0
        else:
            return 1


if __name__ == '__main__':
    setup_stdout_encoding()
    exit(main())
