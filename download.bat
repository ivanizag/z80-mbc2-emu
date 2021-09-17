mkdir sd
powershell -Command "Invoke-WebRequest https://cdn.hackaday.io/files/1599736844284832/S220718-R240620_IOS-Z80-MBC2.zip -OutFile sd.zip"
powershell -Command "Expand-Archive -DestinationPath sd -LiteralPath sd.zip"
del sd.zip
