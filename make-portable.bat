@echo off
chcp 65001 >nul
echo ============================================
echo   制作便携版
echo ============================================
echo.

echo [步骤 1/4] 正在打包应用...
call npm run tauri build
if errorlevel 1 (
    echo 打包失败，请检查错误信息
    pause
    exit /b 1
)

echo.
echo [步骤 2/4] 查找生成的文件...
set "EXE_PATH=src-tauri\target\release\syj_require_list.exe"
if not exist "%EXE_PATH%" (
    echo 未找到 exe 文件: %EXE_PATH%
    pause
    exit /b 1
)

echo.
echo [步骤 3/4] 创建便携版文件夹...
set "PORTABLE_DIR=portable-release"
if exist "%PORTABLE_DIR%" rd /s /q "%PORTABLE_DIR%"
mkdir "%PORTABLE_DIR%"

rem 复制 exe 文件
copy "%EXE_PATH%" "%PORTABLE_DIR%\" >nul

rem 创建 portable.txt 标记文件
echo. > "%PORTABLE_DIR%\portable.txt"

rem 创建 data 文件夹
mkdir "%PORTABLE_DIR%\data"

rem 创建说明文件
(
echo # SYJ Todo List 便携版
echo.
echo ## 使用说明
echo.
echo 1. 直接双击 syj_require_list.exe 运行
echo 2. 所有数据（数据库、图片、配置）都保存在 data 文件夹中
echo 3. 可以将整个文件夹复制到其他电脑使用
echo.
echo ## 首次运行
echo.
echo 首次运行时，如果提示缺少 WebView2，请：
echo - 访问 https://developer.microsoft.com/zh-cn/microsoft-edge/webview2/
echo - 下载并安装 WebView2 运行时
echo.
echo ## 文件夹结构
echo.
echo portable-release/
echo ├── syj_require_list.exe    # 程序主文件
echo ├── portable.txt            # 便携模式标记（请勿删除）
echo ├── data/                   # 数据目录
echo │   ├── todos.db           # 数据库文件
echo │   ├── images/            # 图片文件夹
echo │   └── config.json        # 配置文件
echo └── 使用说明.txt            # 本文件
echo.
echo ## 注意事项
echo.
echo - 请勿删除 portable.txt 文件，否则程序会使用系统 AppData 目录
echo - data 文件夹包含所有用户数据，请定期备份
echo - Windows 10 及以上系统支持
echo.
echo 版本: 0.1.0
) > "%PORTABLE_DIR%\使用说明.txt"

echo.
echo [步骤 4/4] 打包成 ZIP...
set "ZIP_NAME=syj_require_list_portable_v0.1.0.zip"
if exist "%ZIP_NAME%" del /f /q "%ZIP_NAME%"

rem 使用 PowerShell 压缩（Windows 10+ 自带）
powershell -Command "Compress-Archive -Path '%PORTABLE_DIR%\*' -DestinationPath '%ZIP_NAME%' -Force"

echo.
echo ============================================
echo   ✅ 便携版制作完成！
echo ============================================
echo.
echo 📦 便携版文件: %ZIP_NAME%
echo 📁 文件夹: %PORTABLE_DIR%
echo.
echo 使用方法:
echo 1. 解压 %ZIP_NAME% 到任意位置
echo 2. 双击 syj_require_list.exe 运行
echo 3. 所有数据保存在 data 文件夹中
echo.
pause
