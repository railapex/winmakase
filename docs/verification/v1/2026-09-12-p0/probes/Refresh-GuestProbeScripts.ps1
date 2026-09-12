$ErrorActionPreference='Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
foreach($name in @('Capture-P0DialogProof.ps1','DialogProof.Common.psm1','Compare-P0DialogProof.ps1')) { Copy-Item -LiteralPath "C:/WinmakaseSource/$name" -Destination "C:/WinmakaseP0/source/$name" -Force }
