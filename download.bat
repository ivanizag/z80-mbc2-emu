mkdir sd
powershell -Command "Invoke-WebRequest https://cdn.hackaday.io/files/1599736844284832/SD-S220718-R290823-v2.zip -OutFile sd.zip"
powershell -Command "Expand-Archive -DestinationPath sd -LiteralPath sd.zip"
del sd.zip
