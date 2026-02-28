@echo off
chcp 65001 > nul
setlocal enabledelayedexpansion

REM 切换到脚本所在目录
cd /d "%~dp0"

REM 输出模式配置: console=仅控制台, log=仅日志, both=同时输出到控制台和日志
set "OUTPUT_MODE=console"
set "LOG_FILE=%~dp0run-classify.log"

REM 解析命令行参数（包括输出模式）
:parse_args_initial
if "%~1"=="" goto :init_done
if "%~1"=="--output-mode" (
    set "OUTPUT_MODE=%~2"
    shift
    shift
    goto :parse_args_initial
) else if "%~1"=="--log" (
    set "OUTPUT_MODE=log"
    shift
    goto :parse_args_initial
) else if "%~1"=="--both" (
    set "OUTPUT_MODE=both"
    shift
    goto :parse_args_initial
)
shift
goto :parse_args_initial
:init_done

REM 根据输出模式初始化
if "%OUTPUT_MODE%"=="log" (
    REM 仅日志模式：清空日志文件
    echo. > "%LOG_FILE%"
    echo ============ 文件夹分类器执行脚本 ============ >> "%LOG_FILE%"
    echo 启动时间: %date% %time% >> "%LOG_FILE%"
    echo 工作目录: %CD% >> "%LOG_FILE%"
    echo 输出模式: 仅日志 >> "%LOG_FILE%"
    echo. >> "%LOG_FILE%"
) else if "%OUTPUT_MODE%"=="both" (
    REM 同时输出模式：清空日志文件
    echo. > "%LOG_FILE%"
    echo ============ 文件夹分类器执行脚本 ============ >> "%LOG_FILE%"
    echo 启动时间: %date% %time% >> "%LOG_FILE%"
    echo 工作目录: %CD% >> "%LOG_FILE%"
    echo 输出模式: 控制台+日志 >> "%LOG_FILE%"
    echo. >> "%LOG_FILE%"
)

REM 默认参数值
set "TARGET_DIR=D:\Download\pic"
set "CONFIG_FILE=D:\Code\AI\py\classify\backup\classfy.json"
set "OUTPUT_DIR=D:\Download\classify"
set "PARALLEL_WORKERS=4"
set "IO_WORKERS=6"
set "EXPERIMENTAL_MODE=--experimental"
set "NO_INTERACTION="
set "FOLDER1=D:\Download\classify"
set "FOLDER2=E:\classify"
set "FOLDER3=G:\classify"
set "FOLDER4="
set "MERGE_WORKERS=6"
set "DEBUG_MODE="

REM 显示帮助信息
if "%~1"=="/" goto :show_help
if "%~1"=="-h" goto :show_help
if "%~1"=="--help" goto :show_help

REM 解析命令行参数
:parse_args
if "%~1"=="" goto :run_scripts
if "%~1"=="--target" (
    set "TARGET_DIR=%~2"
    shift
) else if "%~1"=="-d" (
    set "TARGET_DIR=%~2"
    shift
) else if "%~1"=="--config" (
    set "CONFIG_FILE=%~2"
    shift
) else if "%~1"=="-c" (
    set "CONFIG_FILE=%~2"
    shift
) else if "%~1"=="--output" (
    set "OUTPUT_DIR=%~2"
    shift
) else if "%~1"=="-o" (
    set "OUTPUT_DIR=%~2"
    shift
) else if "%~1"=="--experimental" (
    set "EXPERIMENTAL_MODE=--experimental"
) else if "%~1"=="-e" (
    set "EXPERIMENTAL_MODE=--experimental"
) else if "%~1"=="--parallel" (
    set "PARALLEL_WORKERS=%~2"
    shift
) else if "%~1"=="-p" (
    set "PARALLEL_WORKERS=%~2"
    shift
) else if "%~1"=="--io-workers" (
    set "IO_WORKERS=%~2"
    shift
) else if "%~1"=="--no-interaction" (
    set "NO_INTERACTION=--no-interaction"
) else if "%~1"=="--folder1" (
    set "FOLDER1=%~2"
    shift
) else if "%~1"=="--folder2" (
    set "FOLDER2=%~2"
    shift
) else if "%~1"=="--folder3" (
    set "FOLDER3=%~2"
    shift
) else if "%~1"=="--folder4" (
    set "FOLDER4=%~2"
    shift
) else if "%~1"=="--merge-workers" (
    set "MERGE_WORKERS=%~2"
    shift
) else if "%~1"=="--debug" (
    set "DEBUG_MODE=--debug"
)
shift
goto :parse_args

:run_scripts
echo.
echo ============ 文件夹分类器执行脚本 ============
echo.

REM 根据输出模式记录参数配置
if "%OUTPUT_MODE%"=="console" (
    echo 输出模式: 仅控制台
) else if "%OUTPUT_MODE%"=="log" (
    echo 开始执行... >> "%LOG_FILE%"
    echo 参数配置: >> "%LOG_FILE%"
    echo   TARGET_DIR: !TARGET_DIR! >> "%LOG_FILE%"
    echo   CONFIG_FILE: !CONFIG_FILE! >> "%LOG_FILE%"
    echo   OUTPUT_DIR: !OUTPUT_DIR! >> "%LOG_FILE%"
    echo   FOLDER1: !FOLDER1! >> "%LOG_FILE%"
    echo   FOLDER2: !FOLDER2! >> "%LOG_FILE%"
    echo. >> "%LOG_FILE%"
) else if "%OUTPUT_MODE%"=="both" (
    echo 输出模式: 控制台+日志
    echo 开始执行... >> "%LOG_FILE%"
    echo 参数配置: >> "%LOG_FILE%"
    echo   TARGET_DIR: !TARGET_DIR! >> "%LOG_FILE%"
    echo   CONFIG_FILE: !CONFIG_FILE! >> "%LOG_FILE%"
    echo   OUTPUT_DIR: !OUTPUT_DIR! >> "%LOG_FILE%"
    echo   FOLDER1: !FOLDER1! >> "%LOG_FILE%"
    echo   FOLDER2: !FOLDER2! >> "%LOG_FILE%"
    echo. >> "%LOG_FILE%"
)

REM 检查是否提供了目标目录
if "%TARGET_DIR%"=="" (
    echo 错误: 必须指定目标目录，使用 --target 或 -d 参数
    echo 例如: %~nx0 --target "C:\MyFolder" --folder1 "C:\Folder1" --folder2 "C:\Folder2" --folder3 "C:\Folder3" --folder4 "C:\Folder4"
    echo.
    goto :show_help
)

REM 检查是否提供了至少两个合并目录用于第一次合并
if "%FOLDER1%"=="" (
    echo 错误: 必须指定第一个合并目录，使用 --folder1 参数
    echo.
    goto :show_help
)
if "%FOLDER2%"=="" (
    echo 错误: 必须指定第二个合并目录，使用 --folder2 参数
    echo.
    goto :show_help
)

echo.

REM 执行 Set 文件夹合并预处理
echo 正在执行 Set 文件夹合并预处理...
echo 参数: "!TARGET_DIR!" --yes

if "%OUTPUT_MODE%"=="console" (
    python "%~dp0merge_set_folders.py" "!TARGET_DIR!" --yes
    set "MERGE_SET_RESULT=!ERRORLEVEL!"
) else if "%OUTPUT_MODE%"=="log" (
    echo 正在执行 Set 文件夹合并预处理... >> "%LOG_FILE%"
    echo 参数: "!TARGET_DIR!" --yes >> "%LOG_FILE%"
    python "%~dp0merge_set_folders.py" "!TARGET_DIR!" --yes >> "%LOG_FILE%" 2>&1
    set "MERGE_SET_RESULT=!ERRORLEVEL!"
    echo Set 文件夹合并预处理退出码: !MERGE_SET_RESULT! >> "%LOG_FILE%"
    echo. >> "%LOG_FILE%"
) else if "%OUTPUT_MODE%"=="both" (
    echo 正在执行 Set 文件夹合并预处理... >> "%LOG_FILE%"
    echo 参数: "!TARGET_DIR!" --yes >> "%LOG_FILE%"
    python "%~dp0merge_set_folders.py" "!TARGET_DIR!" --yes > temp_output.txt 2>&1
    type temp_output.txt
    type temp_output.txt >> "%LOG_FILE%"
    set "MERGE_SET_RESULT=!ERRORLEVEL!"
    del temp_output.txt
    echo Set 文件夹合并预处理退出码: !MERGE_SET_RESULT! >> "%LOG_FILE%"
    echo. >> "%LOG_FILE%"
)

if !MERGE_SET_RESULT! neq 0 (
    echo 错误: Set 文件夹合并预处理执行失败
    if "%OUTPUT_MODE%"=="log" (
        echo 错误: Set 文件夹合并预处理执行失败 >> "%LOG_FILE%"
    ) else if "%OUTPUT_MODE%"=="both" (
        echo 错误: Set 文件夹合并预处理执行失败 >> "%LOG_FILE%"
    )
    echo.
    pause
    goto :end
)

echo.
echo Set 文件夹合并预处理完成
echo.

REM 构建分类器的参数
echo 正在执行文件夹分类器...
echo 参数: --target "!TARGET_DIR!" --config "!CONFIG_FILE!" --parallel !PARALLEL_WORKERS! --io-workers !IO_WORKERS! --output "!OUTPUT_DIR!" !EXPERIMENTAL_MODE! !NO_INTERACTION!

if "%OUTPUT_MODE%"=="console" (
    python "%~dp0folder_classifier_v5_improved2.py" --target "!TARGET_DIR!" --config "!CONFIG_FILE!" --parallel !PARALLEL_WORKERS! --io-workers !IO_WORKERS! --output "!OUTPUT_DIR!" !EXPERIMENTAL_MODE! !NO_INTERACTION!
    set "CLASSIFIER_RESULT=!ERRORLEVEL!"
) else if "%OUTPUT_MODE%"=="log" (
    echo 正在执行文件夹分类器... >> "%LOG_FILE%"
    echo 参数: --target "!TARGET_DIR!" --config "!CONFIG_FILE!" --parallel !PARALLEL_WORKERS! --io-workers !IO_WORKERS! --output "!OUTPUT_DIR!" !EXPERIMENTAL_MODE! !NO_INTERACTION! >> "%LOG_FILE%"
    python "%~dp0folder_classifier_v5_improved2.py" --target "!TARGET_DIR!" --config "!CONFIG_FILE!" --parallel !PARALLEL_WORKERS! --io-workers !IO_WORKERS! --output "!OUTPUT_DIR!" !EXPERIMENTAL_MODE! !NO_INTERACTION! >> "%LOG_FILE%" 2>&1
    set "CLASSIFIER_RESULT=!ERRORLEVEL!"
    echo 文件夹分类器退出码: !CLASSIFIER_RESULT! >> "%LOG_FILE%"
    echo. >> "%LOG_FILE%"
) else if "%OUTPUT_MODE%"=="both" (
    echo 正在执行文件夹分类器... >> "%LOG_FILE%"
    echo 参数: --target "!TARGET_DIR!" --config "!CONFIG_FILE!" --parallel !PARALLEL_WORKERS! --io-workers !IO_WORKERS! --output "!OUTPUT_DIR!" !EXPERIMENTAL_MODE! !NO_INTERACTION! >> "%LOG_FILE%"
    python "%~dp0folder_classifier_v5_improved2.py" --target "!TARGET_DIR!" --config "!CONFIG_FILE!" --parallel !PARALLEL_WORKERS! --io-workers !IO_WORKERS! --output "!OUTPUT_DIR!" !EXPERIMENTAL_MODE! !NO_INTERACTION! > temp_output.txt 2>&1
    type temp_output.txt
    type temp_output.txt >> "%LOG_FILE%"
    set "CLASSIFIER_RESULT=!ERRORLEVEL!"
    del temp_output.txt
    echo 文件夹分类器退出码: !CLASSIFIER_RESULT! >> "%LOG_FILE%"
    echo. >> "%LOG_FILE%"
)

if !CLASSIFIER_RESULT! neq 0 (
    echo 错误: 文件夹分类器执行失败
    if "%OUTPUT_MODE%"=="log" (
        echo 错误: 文件夹分类器执行失败 >> "%LOG_FILE%"
    ) else if "%OUTPUT_MODE%"=="both" (
        echo 错误: 文件夹分类器执行失败 >> "%LOG_FILE%"
    )
    echo.
    pause
    goto :end
)

echo.
echo 文件夹分类器执行完成
echo.

REM 执行文件夹合并器
echo 正在执行文件夹合并器...

REM 构建合并器参数（根据 FOLDER4 是否为空动态构建）
if "%FOLDER4%"=="" (
    echo 参数: "!FOLDER1!" "!FOLDER2!" "!FOLDER3!" --workers !MERGE_WORKERS! !DEBUG_MODE!
) else (
    echo 参数: "!FOLDER1!" "!FOLDER2!" "!FOLDER3!" "!FOLDER4!" --workers !MERGE_WORKERS! !DEBUG_MODE!
)

if "%OUTPUT_MODE%"=="console" (
    if "%FOLDER4%"=="" (
        python "%~dp0mergeClassifierSimple.py" "!FOLDER1!" "!FOLDER2!" "!FOLDER3!" --workers !MERGE_WORKERS! !DEBUG_MODE!
    ) else (
        python "%~dp0mergeClassifierSimple.py" "!FOLDER1!" "!FOLDER2!" "!FOLDER3!" "!FOLDER4!" --workers !MERGE_WORKERS! !DEBUG_MODE!
    )
    set "MERGER_RESULT=!ERRORLEVEL!"
) else if "%OUTPUT_MODE%"=="log" (
    echo 正在执行文件夹合并器... >> "%LOG_FILE%"
    if "%FOLDER4%"=="" (
        echo 参数: "!FOLDER1!" "!FOLDER2!" "!FOLDER3!" --workers !MERGE_WORKERS! !DEBUG_MODE! >> "%LOG_FILE%"
        python "%~dp0mergeClassifierSimple.py" "!FOLDER1!" "!FOLDER2!" "!FOLDER3!" --workers !MERGE_WORKERS! !DEBUG_MODE! >> "%LOG_FILE%" 2>&1
    ) else (
        echo 参数: "!FOLDER1!" "!FOLDER2!" "!FOLDER3!" "!FOLDER4!" --workers !MERGE_WORKERS! !DEBUG_MODE! >> "%LOG_FILE%"
        python "%~dp0mergeClassifierSimple.py" "!FOLDER1!" "!FOLDER2!" "!FOLDER3!" "!FOLDER4!" --workers !MERGE_WORKERS! !DEBUG_MODE! >> "%LOG_FILE%" 2>&1
    )
    set "MERGER_RESULT=!ERRORLEVEL!"
    echo 文件夹合并器退出码: !MERGER_RESULT! >> "%LOG_FILE%"
    echo. >> "%LOG_FILE%"
) else if "%OUTPUT_MODE%"=="both" (
    echo 正在执行文件夹合并器... >> "%LOG_FILE%"
    if "%FOLDER4%"=="" (
        echo 参数: "!FOLDER1!" "!FOLDER2!" "!FOLDER3!" --workers !MERGE_WORKERS! !DEBUG_MODE! >> "%LOG_FILE%"
        python "%~dp0mergeClassifierSimple.py" "!FOLDER1!" "!FOLDER2!" "!FOLDER3!" --workers !MERGE_WORKERS! !DEBUG_MODE! > temp_output.txt 2>&1
    ) else (
        echo 参数: "!FOLDER1!" "!FOLDER2!" "!FOLDER3!" "!FOLDER4!" --workers !MERGE_WORKERS! !DEBUG_MODE! >> "%LOG_FILE%"
        python "%~dp0mergeClassifierSimple.py" "!FOLDER1!" "!FOLDER2!" "!FOLDER3!" "!FOLDER4!" --workers !MERGE_WORKERS! !DEBUG_MODE! > temp_output.txt 2>&1
    )
    type temp_output.txt
    type temp_output.txt >> "%LOG_FILE%"
    set "MERGER_RESULT=!ERRORLEVEL!"
    del temp_output.txt
    echo 文件夹合并器退出码: !MERGER_RESULT! >> "%LOG_FILE%"
    echo. >> "%LOG_FILE%"
)

if !MERGER_RESULT! neq 0 (
    echo 警告: 文件夹合并器执行失败
    if "%OUTPUT_MODE%"=="log" (
        echo 警告: 文件夹合并器执行失败 >> "%LOG_FILE%"
    ) else if "%OUTPUT_MODE%"=="both" (
        echo 警告: 文件夹合并器执行失败 >> "%LOG_FILE%"
    )
    echo.
    pause
) else (
    echo 文件夹合并器执行完成
    if "%OUTPUT_MODE%"=="log" (
        echo 文件夹合并器执行完成 >> "%LOG_FILE%"
    ) else if "%OUTPUT_MODE%"=="both" (
        echo 文件夹合并器执行完成 >> "%LOG_FILE%"
    )
)

echo.

echo ============ 所有任务执行完成 ============
echo.

if "%OUTPUT_MODE%"=="log" (
    echo ============ 所有任务执行完成 ============ >> "%LOG_FILE%"
    echo 结束时间: %date% %time% >> "%LOG_FILE%"
    echo. >> "%LOG_FILE%"
    echo 日志已保存到: %LOG_FILE%
) else if "%OUTPUT_MODE%"=="both" (
    echo ============ 所有任务执行完成 ============ >> "%LOG_FILE%"
    echo 结束时间: %date% %time% >> "%LOG_FILE%"
    echo. >> "%LOG_FILE%"
    echo 日志已保存到: %LOG_FILE%
)

goto :end

:show_help
echo.
echo 用法: %~nx0 [选项]
echo.
echo 功能: 先执行文件夹分类器，再执行两次文件夹合并器
echo.
echo 输出模式选项:
echo   --output-mode       输出模式: console(默认), log(仅日志), both(同时输出)
echo   --log               简写，等同于 --output-mode log
echo   --both              简写，等同于 --output-mode both
echo.
echo 必需参数:
echo   --target, -d        目标文件夹路径 (用于分类器)
echo   --folder1           第一个根文件夹路径 (用于第一次合并器)
echo   --folder2           第二个根文件夹路径 (用于第一次合并器)
echo.
echo 可选参数 (用于第二次合并器):
echo   --folder3           第三个根文件夹路径 (用于第二次合并器)
echo   --folder4           第四个根文件夹路径 (用于第二次合并器)
echo.
echo 分类器可选参数:
echo   --config, -c        配置文件路径 (默认: classification_rules.json)
echo   --output, -o        输出根目录
echo   --experimental, -e  实验模式，只输出结果不执行文件移动
echo   --parallel, -p      分类工作线程数 (默认: 4)
echo   --io-workers        IO工作线程数 (默认: 2)
echo   --no-interaction    禁用用户交互
echo.
echo 合并器可选参数:
echo   --merge-workers     合并工作线程数 (默认: 4)
echo   --debug             启用调试日志
echo.
echo 示例:
echo   %~nx0                                          (默认控制台输出)
echo   %~nx0 --log                                    (仅日志输出)
echo   %~nx0 --both                                   (同时输出到控制台和日志)
echo   %~nx0 --output-mode both --target "C:\MyFiles" --folder1 "C:\Folder1" --folder2 "C:\Folder2"
echo   %~nx0 --target "C:\MyFiles" --folder1 "C:\Folder1" --folder2 "C:\Folder2" --merge-workers 8
echo.
pause
goto :end

:end
endlocal
