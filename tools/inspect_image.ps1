Add-Type -AssemblyName System.Drawing
$filePath = 'C:\Users\rafy2\Downloads\Gemini_Generated_Image_wsedjqwsedjqwsed.jpg'
$img = [System.Drawing.Bitmap]::FromFile($filePath)
Write-Host "Width:  $($img.Width)"
Write-Host "Height: $($img.Height)"
$img.Dispose()
