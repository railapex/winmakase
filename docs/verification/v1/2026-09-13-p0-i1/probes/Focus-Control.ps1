Add-Type -AssemblyName System.Windows.Forms
$form=New-Object Windows.Forms.Form
$form.Text='Winmakase I1 focus control'
$form.Width=420
$form.Height=220
$form.StartPosition='CenterScreen'
[Windows.Forms.Application]::Run($form)
