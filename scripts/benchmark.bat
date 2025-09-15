@echo off
REM Shimmy Lightweight Performance Benchmark for Windows
REM Zero dependencies - uses only system tools

setlocal enabledelayedexpansion

set SHIMMY_URL=http://localhost:11434
set OUTPUT_FILE=benchmark_results.json
set NUM_REQUESTS=10
set MODEL_NAME=

:parse_args
if "%1"=="--url" (
    set SHIMMY_URL=%2
    shift
    shift
    goto parse_args
)
if "%1"=="--model" (
    set MODEL_NAME=%2
    shift
    shift
    goto parse_args
)
if "%1"=="--requests" (
    set NUM_REQUESTS=%2
    shift
    shift
    goto parse_args
)
if "%1"=="--output" (
    set OUTPUT_FILE=%2
    shift
    shift
    goto parse_args
)
if "%1"=="--help" (
    echo Usage: %0 [OPTIONS]
    echo Options:
    echo   --url URL       Shimmy server URL ^(default: http://localhost:11434^)
    echo   --model NAME    Model name to test ^(auto-detect if not specified^)
    echo   --requests N    Number of test requests ^(default: 10^)
    echo   --output FILE   Output file ^(default: benchmark_results.json^)
    echo   --help         Show this help
    exit /b 0
)
if not "%1"=="" (
    shift
    goto parse_args
)

echo 🚀 Shimmy Lightweight Benchmark
echo Target: %SHIMMY_URL%
echo Requests: %NUM_REQUESTS%

REM Check if curl is available
curl --version >nul 2>&1
if errorlevel 1 (
    echo ❌ curl is required but not found
    exit /b 1
)

REM Check if Shimmy is running
curl -s %SHIMMY_URL%/health >nul 2>&1
if errorlevel 1 (
    echo ❌ Cannot connect to Shimmy at %SHIMMY_URL%
    exit /b 1
)

echo ✅ Shimmy is running

REM Get model name if not specified
if "%MODEL_NAME%"=="" (
    for /f "tokens=2 delims=:" %%a in ('curl -s %SHIMMY_URL%/v1/models ^| findstr /r "\"id\":"') do (
        set MODEL_NAME=%%a
        set MODEL_NAME=!MODEL_NAME:"=!
        set MODEL_NAME=!MODEL_NAME:,=!
        set MODEL_NAME=!MODEL_NAME: =!
        goto model_found
    )
    :model_found
    if "!MODEL_NAME!"=="" (
        echo ❌ No models found
        exit /b 1
    )
)

echo 📦 Using model: %MODEL_NAME%

REM System info
echo 📊 System Metrics:
for /f "tokens=2" %%a in ('wmic computersystem get TotalPhysicalMemory /value ^| findstr "="') do set TOTAL_MEM=%%a
for /f "tokens=2" %%a in ('wmic cpu get NumberOfCores /value ^| findstr "="') do set CPU_CORES=%%a

echo   CPU Cores: %CPU_CORES%
set /a MEM_GB=%TOTAL_MEM:~0,-9%
echo   Total Memory: %MEM_GB%GB

REM Check for NVIDIA GPU
nvidia-smi --query-gpu=name,memory.total,utilization.gpu,power.draw --format=csv,noheader,nounits >nul 2>&1
if not errorlevel 1 (
    for /f "tokens=1-4 delims=," %%a in ('nvidia-smi --query-gpu=name,memory.total,utilization.gpu,power.draw --format=csv,noheader,nounits') do (
        echo   GPU: %%a
        echo   GPU Memory: %%b MB
        echo   GPU Util: %%c%%
        echo   GPU Power: %%d W
    )
)

REM Performance test
echo.
echo 🧪 Running performance test...

set SUCCESSFUL=0
set FAILED=0
set TOTAL_TIME=0

for /L %%i in (1,1,%NUM_REQUESTS%) do (
    echo Request %%i/%NUM_REQUESTS%...
    
    REM Create JSON payload
    echo {"model":"%MODEL_NAME%","messages":[{"role":"user","content":"Hello, how are you?"}],"max_tokens":100,"temperature":0.7} > temp_request.json
    
    REM Make request with timing
    set START_TIME=!time!
    curl -s -X POST %SHIMMY_URL%/v1/chat/completions -H "Content-Type: application/json" -d @temp_request.json > temp_response.json 2>nul
    set END_TIME=!time!
    
    REM Check if request was successful
    findstr "choices" temp_response.json >nul 2>&1
    if not errorlevel 1 (
        set /a SUCCESSFUL+=1
        echo   ✅ Success
    ) else (
        set /a FAILED+=1
        echo   ❌ Failed
    )
    
    timeout /t 1 /nobreak >nul
)

REM Calculate results
set /a SUCCESS_RATE=(%SUCCESSFUL% * 100) / %NUM_REQUESTS%

echo.
echo 📊 Results:
echo   Success Rate: %SUCCESS_RATE%%%
echo   Total Requests: %NUM_REQUESTS%
echo   Successful: %SUCCESSFUL%
echo   Failed: %FAILED%

REM Generate JSON output
(
echo {
echo   "timestamp": "%date% %time%",
echo   "hostname": "%COMPUTERNAME%",
echo   "os": "Windows",
echo   "cpu_cores": %CPU_CORES%,
echo   "memory_total_gb": %MEM_GB%,
echo   "shimmy_url": "%SHIMMY_URL%",
echo   "model_name": "%MODEL_NAME%",
echo   "benchmark_results": {
echo     "total_requests": %NUM_REQUESTS%,
echo     "successful_requests": %SUCCESSFUL%,
echo     "failed_requests": %FAILED%,
echo     "success_rate": %SUCCESS_RATE%
echo   }
echo }
) > %OUTPUT_FILE%

REM Cleanup
if exist temp_request.json del temp_request.json
if exist temp_response.json del temp_response.json

echo.
echo 💾 Results saved to: %OUTPUT_FILE%