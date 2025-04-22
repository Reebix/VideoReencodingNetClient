@ECHO OFF
ECHO StartLoop.bat" is running...
REM Diese Batch-Datei startet die VideoReencodingNetClient.exe in einer Schleife
REM Sie wird weiterlaufen, bis keine Dateien mehr zu verarbeiten sind

:loop
VideoReencodingNetClient.exe | find "No file to process" >nul
IF %ERRORLEVEL% EQU 0 (
    ECHO No file to process" gefunden. Beende die Schleife...
    PAUSE
    EXIT /B
)
ECHO VideoReencodingNetClient.exe hat die Verarbeitung abgeschlossen. Neustart...
goto loop