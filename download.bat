mkdir sd
powershell -Command "Invoke-WebRequest https://web.archive.org/web/20220220074001if_/https://cdn.hackaday.io/files/1599736844284832/SD-S220718-R240620-v1.zip -OutFile sd.zip"
powershell -Command "Expand-Archive -DestinationPath sd -LiteralPath sd.zip"
del sd.zip
