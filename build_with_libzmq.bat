echo on
SetLocal EnableDelayedExpansion

REM This is the recommended way to choose the toolchain version, according to
REM Appveyor's documentation.
SET PATH=C:\Program Files (x86)\MSBuild\%TOOLCHAIN_VERSION%\Bin;%PATH%

set VCVARSALL="C:\Program Files (x86)\Microsoft Visual Studio %TOOLCHAIN_VERSION%\VC\vcvarsall.bat"
set MSVCYEAR=vs2015
set MSVCVERSION=v140

if [%Platform%] NEQ [x64] goto win32
set TARGET_ARCH=x86_64
set TARGET_PROGRAM_FILES=%ProgramFiles%
set SODIUM_PLATFORM=X64
set CMAKE_GENERATOR="Visual Studio 14 2015 Win64"
call %VCVARSALL% amd64
if %ERRORLEVEL% NEQ 0 exit 1
goto download

:win32
echo on
if [%Platform%] NEQ [Win32] exit 1
set TARGET_ARCH=i686
set TARGET_PROGRAM_FILES=%ProgramFiles(x86)%
set SODIUM_PLATFORM=x86
set CMAKE_GENERATOR="Visual Studio 14 2015"
call %VCVARSALL% amd64_x86
if %ERRORLEVEL% NEQ 0 exit 1
goto download

:download
REM vcvarsall turns echo off
echo on

echo Installing libsodium
set LIBSODIUM_DIR=C:\projects\libsodium
git clone --branch stable --depth 1 --quiet "https://github.com/jedisct1/libsodium.git" %LIBSODIUM_DIR%
if %ERRORLEVEL% NEQ 0 (
  echo cloning libsodium failed
  exit 1
)

msbuild /v:minimal /maxcpucount:%NUMBER_OF_PROCESSORS% /p:Configuration=%Configuration%DLL %LIBSODIUM_DIR%\builds\msvc\%MSVCYEAR%\libsodium\libsodium.vcxproj
if %ERRORLEVEL% NEQ 0 (
  echo building libsodium failed
  exit 1
)

set SODIUM_LIBRARY_DIR="%LIBSODIUM_DIR%\bin\%SODIUM_PLATFORM%\%Configuration%\%MSVCVERSION%\dynamic"
set SODIUM_INCLUDE_DIR="%LIBSODIUM_DIR%\src\libsodium\include"
move "%SODIUM_LIBRARY_DIR%\libsodium.lib" "%SODIUM_LIBRARY_DIR%\sodium.lib"
if %ERRORLEVEL% NEQ 0 exit 1

set PATH=%SODIUM_LIBRARY_DIR%;%PATH%

echo Installing libzmq
set LIBZMQ_SOURCEDIR=C:\projects\libzmq
git clone --branch v4.2.0 --depth 1 --quiet https://github.com/zeromq/libzmq.git "%LIBZMQ_SOURCEDIR%"
if %ERRORLEVEL% NEQ 0 (
  echo cloning libzmq failed
  exit 1
)

set LIBZMQ_BUILDDIR=C:\projects\build_libzmq
md "%LIBZMQ_BUILDDIR%"
set ORIGINAL_PATH=%CD%
cd "%LIBZMQ_BUILDDIR%"

cmake -D CMAKE_INCLUDE_PATH="%SODIUM_INCLUDE_DIR%" -D CMAKE_LIBRARY_PATH="%SODIUM_LIBRARY_DIR%" -D CMAKE_CXX_FLAGS_RELEASE="/MT" -D CMAKE_CXX_FLAGS_DEBUG="/MTd" -G %CMAKE_GENERATOR% %LIBZMQ_SOURCEDIR%
if %ERRORLEVEL% NEQ 0 (
  echo ...configuring libzmq failed
  exit 1
)

msbuild /v:minimal /maxcpucount:%NUMBER_OF_PROCESSORS% /p:Configuration=%Configuration% libzmq.vcxproj
if %ERRORLEVEL% NEQ 0 (
  echo ...building libzmq failed
  exit 1
)

set LIBZMQ_INCLUDE_DIR=%LIBZMQ_SOURCEDIR%\include
set LIBZMQ_LIB_DIR=%LIBZMQ_BUILDDIR%\lib\%Configuration%
move "%LIBZMQ_LIB_DIR%\libzmq-*lib" "%LIBZMQ_LIB_DIR%\zmq.lib"
set PATH=%LIBZMQ_BUILDDIR%\bin\%Configuration%;%PATH%
if %ERRORLEVEL% NEQ 0 exit 1

cd %ORIGINAL_PATH%

set RUST_URL=https://static.rust-lang.org/dist/rust-%RUST%-%TARGET_ARCH%-pc-windows-msvc.msi
echo Downloading %RUST_URL%...
mkdir build
powershell -Command "(New-Object Net.WebClient).DownloadFile('%RUST_URL%', 'build\rust-%RUST%-%TARGET_ARCH%-pc-windows-msvc.msi')"
if %ERRORLEVEL% NEQ 0 (
  echo ...downloading Rust failed.
  exit 1
)

start /wait msiexec /i build\rust-%RUST%-%TARGET_ARCH%-pc-windows-msvc.msi INSTALLDIR="%TARGET_PROGRAM_FILES%\Rust %RUST%" /quiet /qn /norestart
if %ERRORLEVEL% NEQ 0 exit 1

set PATH="%TARGET_PROGRAM_FILES%\Rust %RUST%\bin";%PATH%

if [%Configuration%] == [Release] set CARGO_MODE=--release

link /?
cl /?
rustc --version
cargo --version

set RUST_BACKTRACE=1

cargo build -vv %CARGO_MODE%
if %ERRORLEVEL% NEQ 0 exit 1

cargo test -vv %CARGO_MODE%
if %ERRORLEVEL% NEQ 0 exit 1