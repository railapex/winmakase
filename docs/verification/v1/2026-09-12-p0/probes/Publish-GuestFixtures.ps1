$ErrorActionPreference='Stop'
& C:/WinmakaseP0/source/Publish-P0GuestShortcuts.ps1 -FixtureRoot C:/WinmakaseP0 -DestinationDirectory (Join-Path ([Environment]::GetFolderPath('Programs')) 'Winmakase P0 Fixtures') -ConsentToken DISPOSABLE-WINDOWS-GUEST
