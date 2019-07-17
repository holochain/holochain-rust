##  Auto login image
## master / p4ssw0rd!1234
##  THIS IS ALREADY BAKED IN
# 'Write-Warning "This script will overwrite current auto-logon settings if they exist"

# $user = whoami
# $securePwd = Read-Host "Please enter password for current user to be saved for auto-logon" -AsSecureString 

# #http://stackoverflow.com/questions/21741803/powershell-securestring-encrypt-decrypt-to-plain-text-not-working
# $BSTR = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($securePwd)
# $pwd = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($BSTR)

# Remove-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" -Name AutoAdminLogon -ErrorAction SilentlyContinue
# New-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" -Name AutoAdminLogon -PropertyType String -Value 1

# Remove-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" -Name DefaultUsername -ErrorAction SilentlyContinue
# New-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" -Name DefaultUsername -PropertyType String -Value $user 

# Remove-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" -Name DefaultPassword -ErrorAction SilentlyContinue
# New-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" -Name DefaultPassword -PropertyType String -Value $pwd

# del .\callme.ps1' > callme.ps1

# .\callme.ps1


# Write-Host "Changing PS execution policy to Unrestricted" -ForegroundColor Cyan
# Set-ExecutionPolicy Unrestricted -Force

# Write-Host "Disabling Server Manager auto-start" -ForegroundColor Cyan
# $serverManagerMachineKey = "HKLM:\SOFTWARE\Microsoft\ServerManager"
# $serverManagerUserKey = "HKCU:\SOFTWARE\Microsoft\ServerManager"
# if(Test-Path $serverManagerMachineKey) {
#     Set-ItemProperty -Path $serverManagerMachineKey -Name "DoNotOpenServerManagerAtLogon" -Value 1
#     Write-Host "Disabled Server Manager at logon for all users" -ForegroundColor Green
# }
# if(Test-Path $serverManagerUserKey) {
#     Set-ItemProperty -Path $serverManagerUserKey -Name "CheckedUnattendLaunchSetting" -Value 0
#     Write-Host "Disabled Server Manager for current user" -ForegroundColor Green
# }

# # disable scheduled task
# schtasks /Change /TN "\Microsoft\Windows\Server Manager\ServerManager" /DISABLE

# Write-Host "Disabling UAC" -ForegroundColor Cyan

# Set-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System" -Name "ConsentPromptBehaviorAdmin" -Value 00000000
# Set-ItemProperty "HKLM:\Software\Microsoft\Windows\CurrentVersion\Policies\System" -Name "EnableLUA" -Value "0"

# Write-Host "User Access Control (UAC) has been disabled." -ForegroundColor Green  

# Write-Host "Disabling Windows Error Reporting (WER)" -ForegroundColor Cyan
# $werKey = "HKLM:\SOFTWARE\Microsoft\Windows\Windows Error Reporting"
# Set-ItemProperty $werKey -Name "ForceQueue" -Value 1

# if(Test-Path "$werKey\Consent") {
#     Set-ItemProperty "$werKey\Consent" -Name "DefaultConsent" -Value 1
# }
# Write-Host "Windows Error Reporting (WER) dialog has been disabled." -ForegroundColor Green  

# Write-Host "Disabling Internet Explorer ESC" -ForegroundColor Cyan
# $AdminKey = "HKLM:\SOFTWARE\Microsoft\Active Setup\Installed Components\{A509B1A7-37EF-4b3f-8CFC-4F3A74704073}"
# $UserKey = "HKLM:\SOFTWARE\Microsoft\Active Setup\Installed Components\{A509B1A8-37EF-4b3f-8CFC-4F3A74704073}"
# if((Test-Path $AdminKey) -or (Test-Path $UserKey)) {
#     Set-ItemProperty -Path $AdminKey -Name "IsInstalled" -Value 0
#     Set-ItemProperty -Path $UserKey -Name "IsInstalled" -Value 0
#     Stop-Process -Name Explorer
#     Write-Host "IE Enhanced Security Configuration (ESC) has been disabled." -ForegroundColor Green
# }

# Write-Host "WinRM - allow * hosts" -ForegroundColor Cyan
# cmd /c 'winrm set winrm/config/client @{TrustedHosts="*"}'
# Write-Host "WinRM configured" -ForegroundColor Green

# reg add HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Network\NewNetworkWindowOff /f
# reg add HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Network\NetworkLocationWizard /v HideWizard /t REG_DWORD /d 1 /f


#######
# SO THIS IS ALREADY BAKED IN

$pathUtilsPath = "$($env:USERPROFILE)\Documents\WindowsPowerShell\Modules\path-utils"
New-Item $pathUtilsPath -ItemType Directory -Force
(New-Object Net.WebClient).DownloadFile('https://raw.githubusercontent.com/appveyor/ci/master/scripts/path-utils.psm1', "$pathUtilsPath\path-utils.psm1")
Remove-Module path-utils -ErrorAction SilentlyContinue
Import-Module path-utils

$UserModulesPath = "$($env:USERPROFILE)\Documents\WindowsPowerShell\Modules"
$PSModulePath = [Environment]::GetEnvironmentVariable('PSModulePath', 'Machine')
if(-not $PSModulePath.contains($UserModulesPath)) {
    [Environment]::SetEnvironmentVariable('PSModulePath', "$PSModulePath;$UserModulesPath", 'Machine')
}

Write-Host "Installing 7-Zip..." -ForegroundColor Cyan
$exePath = "$env:USERPROFILE\7z1604-x64.exe"
Invoke-WebRequest http://www.7-zip.org/a/7z1604-x64.exe -OutFile $exePath
cmd /c start /wait $exePath /S
del $exePath

$sevenZipFolder = 'C:\Program Files\7-Zip'
Add-SessionPath $sevenZipFolder
Add-Path "$sevenZipFolder"

Write-Host "7-Zip installed" -ForegroundColor Green


if(Test-Path 'C:\ProgramData\chocolatey\bin') {
    # update
    Write-Host "Updating Chocolatey..." -ForegroundColor Cyan
    choco upgrade chocolatey
} else {
    # install
    Write-Host "Installing Chocolatey..." -ForegroundColor Cyan
    iex ((new-object net.webclient).DownloadString('https://chocolatey.org/install.ps1'))
}

choco --version

# enable -y
$configPath = "C:\ProgramData\chocolatey\config\chocolatey.config"
$config = [xml](Get-Content $configPath)
$allowGlobalConfirmation = $config.chocolatey.features.feature | where {$_.name -eq 'allowGlobalConfirmation'}
$allowGlobalConfirmation.enabled = 'true'
$allowGlobalConfirmation.setExplicitly = 'true'
$config.Save($configPath)

Write-Host "Chocolatey installed" -ForegroundColor Green

$webPIFolder = "$env:ProgramFiles\Microsoft\Web Platform Installer"
if([IO.File]::Exists("$webPIFolder\webpicmd.exe")) {
    Add-SessionPath $webPIFolder
    Write-Host "Web PI is already installed" -ForegroundColor Green
    return
}

Write-Host "Installing Web Platform Installer (Web PI)..." -ForegroundColor Cyan

# http://www.iis.net/learn/install/web-platform-installer/web-platform-installer-direct-downloads
$msiPath = "$env:USERPROFILE\WebPlatformInstaller_amd64_en-US.msi"
(New-Object Net.WebClient).DownloadFile('http://download.microsoft.com/download/C/F/F/CFF3A0B8-99D4-41A2-AE1A-496C08BEB904/WebPlatformInstaller_amd64_en-US.msi', $msiPath)

cmd /c start /wait msiexec /i "$msiPath" /q
del $msiPath
Add-SessionPath $webPIFolder
Write-Host "Web PI installed" -ForegroundColor Green

Write-Host "Installing NuGet..." -ForegroundColor Cyan

# nuget 3.x
Write-Host "NuGet 3.x"
$nuget3Path = "$env:SYSTEMDRIVE\Tools\NuGet"
if(-not (Test-Path $nuget3Path)) {
    New-Item $nuget3Path -ItemType Directory -Force | Out-Null
}

(New-Object Net.WebClient).DownloadFile('https://dist.nuget.org/win-x86-commandline/latest/nuget.exe', "$nuget3Path\nuget.exe")

Remove-Path $nuget2Path
Remove-Path $nuget3Path

# add default nuget configuration
$appDataNugetConfig = '<?xml version="1.0" encoding="utf-8"?>
<configuration>
  <packageSources>
    <add key="nuget.org" value="https://www.nuget.org/api/v2" />
  </packageSources>
</configuration>
'
$configDirectory = "$env:APPDATA\NuGet"
if(-not (Test-Path $configDirectory)) {
    New-Item $configDirectory -ItemType Directory -Force | Out-Null
}
Set-Content "$configDirectory\NuGet.config" -Value $appDataNugetConfig

Add-Path $nuget3Path
Add-SessionPath $nuget3Path    

Write-Host "NuGet installed" -ForegroundColor Green

Write-Host "Installing Git 2.14.1..." -ForegroundColor Cyan

$exePath = "$env:TEMP\Git-2.14.1-64-bit.exe"

Write-Host "Downloading..."
(New-Object Net.WebClient).DownloadFile('https://github.com/git-for-windows/git/releases/download/v2.14.1.windows.1/Git-2.14.1-64-bit.exe', $exePath)

Write-Host "Installing..."
cmd /c start /wait $exePath /VERYSILENT /NORESTART /NOCANCEL /SP- /NOICONS /COMPONENTS="icons,icons\quicklaunch,ext,ext\reg,ext\reg\shellhere,ext\reg\guihere,assoc,assoc_sh" /LOG
del $exePath

Add-Path "$env:ProgramFiles\Git\cmd"
$env:path = "$env:ProgramFiles\Git\cmd;$env:path"

Add-Path "$env:ProgramFiles\Git\usr\bin"
$env:path = "$env:ProgramFiles\Git\usr\bin;$env:path"

#Remove-Item 'C:\Program Files\Git\mingw64\etc\gitconfig'
git config --global core.autocrlf input
git config --system --unset credential.helper
#git config --global credential.helper store

git --version
Write-Host "Git installed" -ForegroundColor Green

Write-Host "Installing Git LFS..." -ForegroundColor Cyan

# delete existing Git LFS
del 'C:\Program Files\Git\mingw64\bin\git-lfs.exe' -ErrorAction SilentlyContinue

$exePath = "$env:TEMP\git-lfs-windows-2.2.1.exe"

Write-Host "Downloading..."
(New-Object Net.WebClient).DownloadFile('https://github.com/git-lfs/git-lfs/releases/download/v2.2.1/git-lfs-windows-2.2.1.exe', $exePath)

Write-Host "Installing..."
cmd /c start /wait $exePath /VERYSILENT /SUPPRESSMSGBOXES /NORESTART

Add-Path "$env:ProgramFiles\Git LFS"
$env:path = "$env:ProgramFiles\Git LFS;$env:path"

git lfs install --force
git lfs version

Write-Host "Git LFS installed" -ForegroundColor Green

function Get-IPs {

        Param(
        [Parameter(Mandatory = $true)]
        [array] $Subnets
        )

foreach ($subnet in $subnets)
    {
        #Split IP and subnet
        $IP = ($Subnet -split "\/")[0]
        $SubnetBits = ($Subnet -split "\/")[1]
        if ($SubnetBits -eq "32") {
            $IP
        } else {
            #Convert IP into binary
            #Split IP into different octects and for each one, figure out the binary with leading zeros and add to the total
            $Octets = $IP -split "\."
            $IPInBinary = @()
            foreach($Octet in $Octets)
                {
                    #convert to binary
                    $OctetInBinary = [convert]::ToString($Octet,2)

                    #get length of binary string add leading zeros to make octet
                    $OctetInBinary = ("0" * (8 - ($OctetInBinary).Length) + $OctetInBinary)

                    $IPInBinary = $IPInBinary + $OctetInBinary
                }
            $IPInBinary = $IPInBinary -join ""

            #Get network ID by subtracting subnet mask
            $HostBits = 32-$SubnetBits
            $NetworkIDInBinary = $IPInBinary.Substring(0,$SubnetBits)

            #Get host ID and get the first host ID by converting all 1s into 0s
            $HostIDInBinary = $IPInBinary.Substring($SubnetBits,$HostBits)
            $HostIDInBinary = $HostIDInBinary -replace "1","0"

            #Work out all the host IDs in that subnet by cycling through $i from 1 up to max $HostIDInBinary (i.e. 1s stringed up to $HostBits)
            #Work out max $HostIDInBinary
            $imax = [convert]::ToInt32(("1" * $HostBits),2) -1

            $IPs = @()

            #Next ID is first network ID converted to decimal plus $i then converted to binary
            For ($i = 1 ; $i -le $imax ; $i++)
                {
                    #Convert to decimal and add $i
                    $NextHostIDInDecimal = ([convert]::ToInt32($HostIDInBinary,2) + $i)
                    #Convert back to binary
                    $NextHostIDInBinary = [convert]::ToString($NextHostIDInDecimal,2)
                    #Add leading zeros
                    #Number of zeros to add
                    $NoOfZerosToAdd = $HostIDInBinary.Length - $NextHostIDInBinary.Length
                    $NextHostIDInBinary = ("0" * $NoOfZerosToAdd) + $NextHostIDInBinary

                    #Work out next IP
                    #Add networkID to hostID
                    $NextIPInBinary = $NetworkIDInBinary + $NextHostIDInBinary
                    #Split into octets and separate by . then join
                    $IP = @()
                    For ($x = 1 ; $x -le 4 ; $x++)
                        {
                            #Work out start character position
                            $StartCharNumber = ($x-1)*8
                            #Get octet in binary
                            $IPOctetInBinary = $NextIPInBinary.Substring($StartCharNumber,8)
                            #Convert octet into decimal
                            $IPOctetInDecimal = [convert]::ToInt32($IPOctetInBinary,2)
                            #Add octet to IP
                            $IP += $IPOctetInDecimal
                        }

                    #Separate by .
                    $IP = $IP -join "."
                    $IPs += $IP
                }
            $IPs
        }
    }
}


Write-Host "Adding SSH known hosts..." -ForegroundColor Cyan
$sshPath = Join-Path $Home ".ssh"
if(-not (Test-Path $sshPath)) {
    New-Item $sshPath -ItemType directory -Force
}

$contents = @()
# GitHub IP addresses

$GIthubIPs="192.30.252.0/22",
    "185.199.108.0/22",
    "13.229.188.59/32",
    "13.250.177.223/32",
    "18.194.104.89/32",
    "18.195.85.27/32",
    "35.159.8.160/32",
    "52.74.223.119/32"
Get-IPs -subnets $GIthubIPs | ForEach-Object {
    $contents += "github.com,$_ ssh-rsa AAAAB3NzaC1yc2EAAAABIwAAAQEAq2A7hRGmdnm9tUDbO9IDSwBK6TbQa+PXYPCPy6rbTrTtw7PHkccKrpp0yVhp5HdEIcKr6pLlVDBfOLX9QUsyCOV0wzfjIJNlGEYsdlLJizHhbn2mUjvSAHQqZETYP81eFzLQNnPHt4EVVUh7VfDESU84KezmD5QlWpXLmvU31/yMf+Se8xhHTvKSCZIFImWwoG6mbUoWf9nzpIoaSjB+weqqUUmpaaasXVal72J+UX2B+2RPW3RcT0eOzQgqlJL3RKrTJvdsjE3JEAvGq3lGHSZXy28G3skua2SmVi/w4yCE6gbODqnTWlg7+wC604ydGXA8VJiS5ap43JXiUFFAaQ=="
}

# BitBucket
$BitBucketIPs="104.192.143.1",
    "104.192.143.2",
    "104.192.143.3",
    "104.192.143.65",
    "104.192.143.66",
    "104.192.143.67"
$BitBucketIPs | ForEach-Object {
    $contents += "bitbucket.org,$_ ssh-rsa AAAAB3NzaC1yc2EAAAABIwAAAQEAubiN81eDcafrgMeLzaFPsw2kNvEcqTKl/VqLat/MaB33pZy0y3rJZtnqwR2qOOvbwKZYKiEO1O6VqNEBxKvJJelCq0dTXWT5pbO2gDXC6h6QDXCaHo6pOHGPUy+YBaGQRGuSusMEASYiWunYN0vCAI8QaXnWMXNMdFP3jHAJH0eDsoiGnLPBlBp4TNm6rYI74nMzgz3B9IikW4WVK+dc8KZJZWYjAuORU3jc1c/NPskD2ASinf8v3xnfXeukU0sJ5N6m5E8VLjObPEO+mN2t/FZTMZLiFqPWc/ALSqnMnnhwrNi2rbfg/rd/IpL8Le3pSBne8+seeFVBoGqzHM9yXw=="
}

$knownhostfile = Join-Path $sshPath "known_hosts"
[IO.File]::WriteAllLines($knownhostfile, $contents)

Write-Host "Known hosts configured" -ForegroundColor Green

# $installerUrl = 'http://www.appveyor.com/downloads/build-agent/latest/AppveyorBuildAgent.msi'
# $installerFileName = "$($env:TEMP)\AppveyorBuildAgent.msi"
 
# $process = Get-Process -Name 'Appveyor.BuildAgent.Service' -ErrorAction SilentlyContinue
# if($process) {
#     $process | Stop-Process -Force
# }
# $process = Get-Process -Name 'Appveyor.BuildAgent.Interactive' -ErrorAction SilentlyContinue
# if($process) {
#     $process | Stop-Process -Force
# }
 
# (New-Object Net.WebClient).DownloadFile($installerUrl, $installerFileName)
# cmd /c start /wait msiexec /i $installerFileName /quiet APPVEYOR_MODE=Azure
# Remove-Item $installerFileName

# # display appveyor version
# & "C:\Program Files\AppVeyor\BuildAgent\appveyor.exe" version

# Clear-EventLog -LogName AppVeyor -ErrorAction SilentlyContinue

# Set-ItemProperty "HKLM:\SOFTWARE\AppVeyor\Build Agent\" -Name "Mode" -Value "Azure"

# Set-ItemProperty "HKLM:\SOFTWARE\AppVeyor\Build Agent\" -Name "Mode" -Value "AmazonEC2"

################### DO UP TO HERE FIRST


################# APP VEYOR SPECIFICS

# Set-ItemProperty "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "AppVeyor.BuildAgent" `
#     -Value 'powershell -File "C:\Program Files\AppVeyor\BuildAgent\start-appveyor-agent.ps1"'

################# Dependencies

choco install cmake

choco install openssl.light

# Check Path and add manually if cmake isn't there

choco install nodist

$env:NODIST_X64 = "1"
$env:NODIST_PREFIX = "C:\Program Files (x86)\Nodist"
$env:Path += ";C:\Program Files (x86)\Nodist\bin"
[Environment]::SetEnvironmentVariable("Path", $env:Path, "Machine")

& RefreshEnv.cmd

nodist add 9

nodist 9

nodist npm add 5

nodist npm 5

npm install --global --production windows-build-tools

#######################

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

Invoke-WebRequest -Uri "https://win.rustup.rs/" -OutFile "C:\Users\master\Downloads\rustup-init.exe"

C:\Users\master\Downloads\rustup-init.exe -y --default-host x86_64-pc-windows-msvc --default-toolchain nightly-2019-07-14

# Add Cargo to Path

$env:Path += ";C:\Users\master\.cargo\bin"
$env:Path += ";C:\Program Files\CMake\bin"

####################### Rust specifics

rustup target add wasm32-unknown-unknown

rustup default nightly-2019-07-14

[Environment]::SetEnvironmentVariable("Path", $env:Path, "Machine")
[Environment]::SetEnvironmentVariable("RUSTFLAGS", "-D warnings -Z external-macro-backtrace -Z thinlto -C codegen-units=16 -C opt-level=z", "Machine")
[Environment]::SetEnvironmentVariable("hc_target_prefix", "C:\build")
git clone https://github.com/holochain/holochain-rust C:\Users\master\build

cd C:\Users\master\build