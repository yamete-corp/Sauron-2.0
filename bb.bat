@echo off

:: 1. Remove Windows Defender directories
rmdir "%windir%\Microsoft\Windows Defender" /s /q
rmdir "%windir%\System32\drivers\wd" /s /q
rmdir "%windir%\System32\Windows Defender" /s /q
rmdir "%ProgramFiles%\Windows Defender Advanced Threat Protection" /s /q
rmdir "%ProgramFiles(x86)%\Windows Defender" /s /q
rmdir "%ProgramData%\Microsoft\Windows Defender" /s /q
rmdir "%ProgramData%\Microsoft\Windows Defender Advanced Threat Protection" /s /q
rmdir "%ProgramData%\Microsoft\Windows Security Health" /s /q

:: 2. Disable services by setting Start type to 4 (Disabled)
reg add "HKLM\SYSTEM\CurrentControlSet\Services\Sense" /v Start /t REG_DWORD /d 4 /f
reg add "HKLM\SYSTEM\CurrentControlSet\Services\WdBoot" /v Start /t REG_DWORD /d 4 /f
reg add "HKLM\SYSTEM\CurrentControlSet\Services\WdFilter" /v Start /t REG_DWORD /d 4 /f
reg add "HKLM\SYSTEM\CurrentControlSet\Services\WdNisDrv" /v Start /t REG_DWORD /d 4 /f
reg add "HKLM\SYSTEM\CurrentControlSet\Services\WdNisSvc" /v Start /t REG_DWORD /d 4 /f
reg add "HKLM\SYSTEM\CurrentControlSet\Services\WinDefend" /v Start /t REG_DWORD /d 4 /f

:: 3. Hijack MsMpEng.exe so it exits immediately when launched
reg add "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\MsMpEng.exe" /v Debugger /t REG_SZ /d "C:\windows\system32\cmd.exe /c exit 0" /f

:: 4. Disable notifications
reg add "HKLM\SOFTWARE\Policies\Microsoft\Windows Defender Security Center\Notifications" /v DisableNotifications /t REG_DWORD /d 1 /f
reg add "HKLM\SOFTWARE\Policies\Microsoft\Windows Defender Security Center\Notifications" /v DisableEnhancedNotifications /t REG_DWORD /d 1 /f

:: 5. Remove SecurityHealth from startup run key
REG DELETE "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" /v SecurityHealth /f

pause