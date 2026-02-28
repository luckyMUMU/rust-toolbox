"""
批量重命名指定目录下的子文件夹
支持将子文件夹名称中的指定字符串进行替换
默认只替换第一个匹配项，可通过参数控制
支持命令行参数和交互式两种使用方式
"""

import os
import argparse
import sys


def rename_subfolders(directory, old_string, new_string, first_only=True, dry_run=False):
    """
    重命名指定目录下的子文件夹

    :param directory: 目标目录
    :param old_string: 要被替换的字符串
    :param new_string: 替换成的新字符串
    :param first_only: 是否只替换第一个匹配项，默认为True
    :param dry_run: 是否为预览模式（不实际执行重命名），默认为False
    :return: 重命名成功的数量
    """
    renamed_count = 0

    # 获取目录下的所有子目录
    subdirs = [d for d in os.listdir(directory)
               if os.path.isdir(os.path.join(directory, d))]

    for subdir in subdirs:
        # 根据first_only参数决定替换策略
        if first_only:
            # 只替换第一个匹配项
            new_name = subdir.replace(old_string, new_string, 1)
        else:
            # 替换所有匹配项
            new_name = subdir.replace(old_string, new_string)

        # 如果名称发生变化，则进行重命名
        if new_name != subdir:
            old_path = os.path.join(directory, subdir)
            new_path = os.path.join(directory, new_name)

            # 检查目标名称是否已存在，如果存在则添加后缀
            if not dry_run:
                counter = 1
                original_new_path = new_path
                while os.path.exists(new_path):
                    name, ext = os.path.splitext(new_name)
                    new_name = f"{name}-rename{ext}"
                    new_path = os.path.join(directory, new_name)
                    counter += 1
                    # 为了避免无限循环，限制重试次数
                    if counter > 100:
                        print(f"警告: 无法为 '{subdir}' 找到唯一名称，跳过重命名")
                        break
                else:
                    # 如果目标路径不存在，继续重命名
                    pass

            if dry_run:
                print(f"[预览] 将重命名: '{subdir}' -> '{new_name}'")
                # 在预览模式下也要增加计数器
                renamed_count += 1
            else:
                if counter <= 100:  # 只有在未达到重试限制时才执行重命名
                    try:
                        os.rename(old_path, new_path)
                        if new_name != subdir.replace(old_string, new_string, 1 if first_only else -1):
                            print(f"已重命名: '{subdir}' -> '{new_name}' (已解决名称冲突)")
                        else:
                            print(f"已重命名: '{subdir}' -> '{new_name}'")
                        renamed_count += 1
                    except OSError as e:
                        print(f"重命名失败: '{subdir}' -> '{new_name}', 错误: {e}")

    if dry_run:
        print(f"\n[预览] 总共找到 {renamed_count} 个需要重命名的文件夹")
    else:
        print(f"\n成功重命名了 {renamed_count} 个文件夹")

    return renamed_count


def interactive_mode():
    """
    交互式模式：通过用户输入获取参数
    """
    print("=" * 50)
    print("    批量重命名子文件夹工具")
    print("=" * 50)
    print()

    # 获取目标目录
    while True:
        directory = input("请输入目标目录路径: ").strip()
        if not directory:
            print("错误: 目录路径不能为空")
            continue
        if not os.path.isdir(directory):
            print(f"错误: 目录 '{directory}' 不存在，请重新输入")
            continue
        break

    # 获取要被替换的字符串
    while True:
        old_string = input("请输入要被替换的字符串: ").strip()
        if not old_string:
            print("错误: 要被替换的字符串不能为空")
            continue
        break

    # 获取新字符串
    new_string = input("请输入替换成的新字符串 (直接回车表示替换为空): ").strip()

    # 获取替换策略
    print()
    print("替换策略选择:")
    print("  1. 只替换第一个匹配项 (默认)")
    print("  2. 替换所有匹配项")
    choice = input("请选择 (1/2，直接回车默认为1): ").strip()
    first_only = choice != "2"

    # 获取是否预览模式
    print()
    dry_run_input = input("是否启用预览模式？(y/N，直接回车表示否): ").strip().lower()
    dry_run = dry_run_input == 'y'

    # 显示操作摘要
    print()
    print("-" * 50)
    print("操作摘要:")
    print(f"  目标目录: {directory}")
    print(f"  替换 '{old_string}' 为 '{new_string}'")
    print(f"  替换策略: {'只替换第一个匹配项' if first_only else '替换所有匹配项'}")
    print(f"  预览模式: {'是' if dry_run else '否'}")
    print("-" * 50)

    # 确认执行
    if not dry_run:
        confirm = input("确认执行此操作吗？(y/N): ").strip().lower()
        if confirm != 'y':
            print("操作已取消")
            return 0

    # 执行重命名
    print()
    rename_subfolders(
        directory,
        old_string,
        new_string,
        first_only=first_only,
        dry_run=dry_run
    )

    return 0


def main():
    parser = argparse.ArgumentParser(
        description='批量重命名指定目录下的子文件夹',
        usage='%(prog)s [-h] [directory] [old_string] [new_string] [--all] [--dry-run] [--interactive]'
    )
    parser.add_argument('directory', nargs='?', help='目标目录路径')
    parser.add_argument('old_string', nargs='?', help='要被替换的字符串')
    parser.add_argument('new_string', nargs='?', help='替换成的新字符串')
    parser.add_argument('--all', action='store_true',
                        help='替换所有匹配项（默认只替换第一个）')
    parser.add_argument('--dry-run', action='store_true',
                        help='预览模式，不实际执行重命名')
    parser.add_argument('--first-only', action='store_true', default=True,
                        help='只替换第一个匹配项（默认行为）')
    parser.add_argument('-i', '--interactive', action='store_true',
                        help='进入交互式模式')

    args = parser.parse_args()

    # 判断使用哪种模式
    # 如果指定了交互式模式，或者没有提供必需的参数，则进入交互式模式
    if args.interactive or args.directory is None:
        return interactive_mode()

    # 命令行模式
    # 确认目录存在
    if not os.path.isdir(args.directory):
        print(f"错误: 目录 '{args.directory}' 不存在")
        return 1

    # 检查是否同时指定了--all和--first-only
    if args.all and args.first_only:
        print("警告: 同时指定了--all和--first-only，将使用--all选项")

    # 确定替换策略
    first_only = not args.all  # 如果指定了--all，则first_only为False

    # 确认执行操作
    if not args.dry_run:
        print(f"即将在目录 '{args.directory}' 中:")
        print(f"  将 '{args.old_string}' 替换为 '{args.new_string}'")
        print(f"  替换策略: {'只替换第一个匹配项' if first_only else '替换所有匹配项'}")

        confirm = input("确认执行此操作吗？(y/N): ")
        if confirm.lower() != 'y':
            print("操作已取消")
            return 0

    # 执行重命名
    rename_subfolders(
        args.directory,
        args.old_string,
        args.new_string,
        first_only=first_only,
        dry_run=args.dry_run
    )

    return 0


if __name__ == "__main__":
    exit(main())