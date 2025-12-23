# 制作便携版脚本
# 使用方法: 在项目根目录执行 .\make-portable.ps1

Write-Host "开始制作便携版..." -ForegroundColor Green

# 1. 执行打包
Write-Host "步骤 1/4: 正在打包应用..." -ForegroundColor Yellow
npm run tauri build

if ($LASTEXITCODE -ne 0) {
    Write-Host "打包失败，请检查错误信息" -ForegroundColor Red
    exit 1
}

# 2. 查找生成的 exe 文件
Write-Host "步骤 2/4: 查找生成的文件..." -ForegroundColor Yellow
$releaseDir = "src-tauri\target\release"
$exePath = Join-Path $releaseDir "syj_require_list.exe"

if (-not (Test-Path $exePath)) {
    Write-Host "未找到 exe 文件: $exePath" -ForegroundColor Red
    exit 1
}

# 3. 创建便携版文件夹
Write-Host "步骤 3/4: 创建便携版文件夹..." -ForegroundColor Yellow
$portableDir = "portable-release"
if (Test-Path $portableDir) {
    Remove-Item -Recurse -Force $portableDir
}
New-Item -ItemType Directory -Path $portableDir | Out-Null

# 复制 exe 文件
Copy-Item $exePath -Destination $portableDir

# 创建 portable.txt 标记文件（启用便携模式）
New-Item -ItemType File -Path (Join-Path $portableDir "portable.txt") | Out-Null

# 创建 data 文件夹（用于存储数据）
New-Item -ItemType Directory -Path (Join-Path $portableDir "data") | Out-Null

# 创建说明文件
$readme = @"
# SYJ Todo List 便携版

## 使用说明

1. 直接双击 syj_require_list.exe 运行
2. 所有数据（数据库、图片、配置）都保存在 data 文件夹中
3. 可以将整个文件夹复制到其他电脑使用

## 首次运行

首次运行时，如果提示缺少 WebView2，请：
- 访问 https://developer.microsoft.com/zh-cn/microsoft-edge/webview2/
- 下载并安装 WebView2 运行时

## 文件夹结构

```
portable-release/
├── syj_require_list.exe    # 程序主文件
├── portable.txt            # 便携模式标记（请勿删除）
├── data/                   # 数据目录
│   ├── todos.db           # 数据库文件
│   ├── images/            # 图片文件夹
│   └── config.json        # 配置文件
└── 使用说明.txt            # 本文件
```

## 注意事项

- 请勿删除 portable.txt 文件，否则程序会使用系统 AppData 目录
- data 文件夹包含所有用户数据，请定期备份
- Windows 10 及以上系统支持

版本: 0.1.0
"@

Set-Content -Path (Join-Path $portableDir "使用说明.txt") -Value $readme -Encoding UTF8

# 4. 打包成 zip
Write-Host "步骤 4/4: 打包成 ZIP..." -ForegroundColor Yellow
$zipPath = "syj_require_list_portable_v0.1.0.zip"
if (Test-Path $zipPath) {
    Remove-Item -Force $zipPath
}

Compress-Archive -Path "$portableDir\*" -DestinationPath $zipPath

Write-Host "`n✅ 便携版制作完成！" -ForegroundColor Green
Write-Host "📦 便携版文件: $zipPath" -ForegroundColor Cyan
Write-Host "📁 文件夹: $portableDir" -ForegroundColor Cyan
Write-Host "`n使用方法:" -ForegroundColor Yellow
Write-Host "1. 解压 $zipPath 到任意位置" -ForegroundColor White
Write-Host "2. 双击 syj_require_list.exe 运行" -ForegroundColor White
Write-Host "3. 所有数据保存在 data 文件夹中" -ForegroundColor White
