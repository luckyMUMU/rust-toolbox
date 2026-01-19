@echo off
chcp 65001 > nul
set SOURCE_DIR=D:\Download\pic
set TARGET_DIR=D:\Download\classify
set RULES_FILE=D:\Code\AI\py\classify\backup\classfy.json
set RUST_TEMPLATE=D:\Code\AI\rust-tool-v2\examples\templates\interactive-classification-workflow.yaml

@REM echo ========================================================
@REM echo Running Python Classifier (Experimental Mode)
@REM echo ========================================================
@REM python d:\Code\AI\rust-tool-v2\scripts\folder_classifier_v5_improved2.py --target "%SOURCE_DIR%" --output "%TARGET_DIR%" --config "%RULES_FILE%" --experimental --no-interaction

echo.
echo ========================================================
echo Running Rust Workflow (Experimental Mode)
echo ========================================================
cargo run -- workflow execute "%RUST_TEMPLATE%" --param source_directory="%SOURCE_DIR%" --param output_directory="%TARGET_DIR%" --param classification_rules="%RULES_FILE%" --param experimental_mode=true --param enable_user_interaction=true

echo.
echo Done.
pause
