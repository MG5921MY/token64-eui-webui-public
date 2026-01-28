# Docker 构建快速测试脚本
# 用法: .\test-docker-build.ps1

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  IPv6 Token WebUI - Docker 构建测试" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 检查 Docker 是否运行
Write-Host "1. 检查 Docker 状态..." -ForegroundColor Yellow
try {
    docker version | Out-Null
    Write-Host "   ✅ Docker 正在运行" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Docker 未运行，请启动 Docker Desktop" -ForegroundColor Red
    exit 1
}
Write-Host ""

# 清理旧容器和镜像
Write-Host "2. 清理旧容器..." -ForegroundColor Yellow
docker-compose -f docker-compose.local.yml down 2>$null
Write-Host "   ✅ 清理完成" -ForegroundColor Green
Write-Host ""

# 构建镜像
Write-Host "3. 构建 Docker 镜像..." -ForegroundColor Yellow
Write-Host "   （这可能需要 5-10 分钟，请耐心等待）" -ForegroundColor Gray
$buildStart = Get-Date
docker-compose -f docker-compose.local.yml build
if ($LASTEXITCODE -ne 0) {
    Write-Host "   ❌ 构建失败" -ForegroundColor Red
    exit 1
}
$buildEnd = Get-Date
$buildTime = ($buildEnd - $buildStart).TotalSeconds
Write-Host "   ✅ 构建成功（耗时: $([math]::Round($buildTime, 2)) 秒）" -ForegroundColor Green
Write-Host ""

# 查看镜像信息
Write-Host "4. 查看镜像信息..." -ForegroundColor Yellow
$image = docker images ipv6-token-webui --format "{{.Repository}}:{{.Tag}} - {{.Size}}"
Write-Host "   镜像: $image" -ForegroundColor Cyan
Write-Host ""

# 启动容器
Write-Host "5. 启动容器..." -ForegroundColor Yellow
docker-compose -f docker-compose.local.yml up -d
if ($LASTEXITCODE -ne 0) {
    Write-Host "   ❌ 启动失败" -ForegroundColor Red
    exit 1
}
Write-Host "   ✅ 容器已启动" -ForegroundColor Green
Write-Host ""

# 等待服务启动
Write-Host "6. 等待服务启动..." -ForegroundColor Yellow
Start-Sleep -Seconds 3
Write-Host ""

# 查看日志
Write-Host "7. 查看启动日志..." -ForegroundColor Yellow
Write-Host "----------------------------------------" -ForegroundColor Gray
docker-compose -f docker-compose.local.yml logs --tail=20
Write-Host "----------------------------------------" -ForegroundColor Gray
Write-Host ""

# 检查容器状态
Write-Host "8. 检查容器状态..." -ForegroundColor Yellow
$container = docker-compose -f docker-compose.local.yml ps --format json | ConvertFrom-Json
if ($container.State -eq "running") {
    Write-Host "   ✅ 容器运行正常" -ForegroundColor Green
} else {
    Write-Host "   ❌ 容器未运行" -ForegroundColor Red
    exit 1
}
Write-Host ""

# 测试 API
Write-Host "9. 测试 API 连接..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://localhost:5000/api/system/info" -TimeoutSec 5
    if ($response.StatusCode -eq 200) {
        Write-Host "   ✅ API 响应正常" -ForegroundColor Green
    } else {
        Write-Host "   ⚠️  API 响应异常: $($response.StatusCode)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ⚠️  API 连接失败（可能需要 --skip-root-check）" -ForegroundColor Yellow
}
Write-Host ""

# 查看资源使用
Write-Host "10. 查看资源使用..." -ForegroundColor Yellow
docker stats ipv6-token-webui --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}"
Write-Host ""

# 测试总结
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  测试完成！" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "📊 测试结果:" -ForegroundColor White
Write-Host "   - 构建时间: $([math]::Round($buildTime, 2)) 秒" -ForegroundColor Cyan
Write-Host "   - 镜像大小: $image" -ForegroundColor Cyan
Write-Host "   - 容器状态: 运行中" -ForegroundColor Green
Write-Host ""
Write-Host "🌐 访问地址:" -ForegroundColor White
Write-Host "   - WebUI: http://localhost:5000" -ForegroundColor Cyan
Write-Host "   - API: http://localhost:5000/api/system/info" -ForegroundColor Cyan
Write-Host ""
Write-Host "📝 常用命令:" -ForegroundColor White
Write-Host "   - 查看日志: docker-compose -f docker-compose.local.yml logs -f" -ForegroundColor Gray
Write-Host "   - 停止容器: docker-compose -f docker-compose.local.yml down" -ForegroundColor Gray
Write-Host "   - 重启容器: docker-compose -f docker-compose.local.yml restart" -ForegroundColor Gray
Write-Host ""
Write-Host "✨ 现在可以在浏览器中打开 http://localhost:5000 测试 WebUI" -ForegroundColor Green
Write-Host ""

# 询问是否打开浏览器
$open = Read-Host "是否在浏览器中打开 WebUI？(Y/N)"
if ($open -eq "Y" -or $open -eq "y") {
    Start-Process "http://localhost:5000"
}
