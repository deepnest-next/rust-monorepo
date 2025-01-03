#
# The list of VS 2022 components: https://docs.microsoft.com/en-us/visualstudio/install/workload-and-component-ids?view=vs-2022
#

Function InstallVS
{
  Param
  (
    [String] $WorkLoads,
    [String] $Sku,
	[String] $VSBootstrapperURL,
	[String] $ChannelUri
  )

  $exitCode = -1

  try
  {
    Write-Host "Downloading Bootstrapper ..."
    Invoke-WebRequest -Uri $VSBootstrapperURL -OutFile "${env:Temp}\vs_$Sku.exe"

    $FilePath = "${env:Temp}\vs_$Sku.exe"
	$Arguments = ($WorkLoads, '--quiet', '--norestart', '--wait', '--nocache')

	if ($ChannelUri) {
		$Arguments += (
			'--channelUri', $ChannelUri,
			'--installChannelUri', $ChannelUri
		)
	}

    Write-Host "Starting Install ..."
    $process = Start-Process -FilePath $FilePath -ArgumentList $Arguments -Wait -PassThru
    $exitCode = $process.ExitCode

    if ($exitCode -eq 0 -or $exitCode -eq 3010)
    {
      Write-Host -Object 'Installation successful'
      return $exitCode
    }
    else
    {
      Write-Host -Object "Non zero exit code returned by the installation process : $exitCode."

      # this wont work because of log size limitation in extension manager
      # Get-Content $customLogFilePath | Write-Host

      exit $exitCode
    }
  }
  catch
  {
    Write-Host -Object "Failed to install Visual Studio. Check the logs for details in $customLogFilePath"
    Write-Host -Object $_.Exception.Message
    exit -1
  }
}

$WorkLoads = '--add Microsoft.VisualStudio.Component.CoreEditor ' + `
    '--add Microsoft.VisualStudio.Workload.CoreEditor ' + `
    '--add Microsoft.Net.Component.4.8.SDK ' + `
    '--add Microsoft.Net.Component.4.7.2.TargetingPack ' + `
    '--add Microsoft.Net.ComponentGroup.DevelopmentPrerequisites ' + `
    '--add Microsoft.VisualStudio.Component.TypeScript.TSServer ' + `
    '--add Microsoft.VisualStudio.ComponentGroup.WebToolsExtensions ' + `
    '--add Microsoft.VisualStudio.Component.JavaScript.TypeScript ' + `
    '--add Microsoft.VisualStudio.Component.Roslyn.Compiler ' + `
    '--add Microsoft.Component.MSBuild ' + `
    '--add Microsoft.VisualStudio.Component.Roslyn.LanguageServices ' + `
    '--add Microsoft.VisualStudio.Component.TextTemplating ' + `
    '--add Microsoft.VisualStudio.Component.NuGet ' + `
    '--add Microsoft.VisualStudio.Component.SQL.CLR ' + `
    '--add Microsoft.Component.ClickOnce ' + `
    '--add Microsoft.VisualStudio.Component.ManagedDesktop.Core ' + `
    '--add Microsoft.NetCore.Component.Runtime.9.0 ' + `
    '--add Microsoft.NetCore.Component.Runtime.8.0 ' + `
    '--add Microsoft.NetCore.Component.SDK ' + `
    '--add Microsoft.VisualStudio.Component.AppInsights.Tools ' + `
    '--add Microsoft.VisualStudio.Component.DiagnosticTools ' + `
    '--add Microsoft.VisualStudio.Component.Debugger.JustInTime ' + `
    '--add Component.Microsoft.VisualStudio.LiveShare.2022 ' + `
    '--add Microsoft.VisualStudio.Component.IntelliCode ' + `
    '--add Microsoft.NetCore.Component.Runtime.6.0 ' + `
    '--add Microsoft.VisualStudio.Component.VC.CoreIde ' + `
    '--add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 ' + `
    '--add Microsoft.VisualStudio.Component.Graphics.Tools ' + `
    '--add Microsoft.VisualStudio.Component.VC.DiagnosticTools ' + `
    '--add Microsoft.VisualStudio.Component.Windows11SDK.22621 ' + `
    '--add Microsoft.VisualStudio.ComponentGroup.MSIX.Packaging ' + `
    '--add Microsoft.VisualStudio.ComponentGroup.WindowsAppSDK.Cs ' + `
    '--add Microsoft.VisualStudio.Component.VC.ATL ' + `
    '--add Microsoft.VisualStudio.Component.VC.ATLMFC ' + `
    '--add Microsoft.VisualStudio.Component.VC.Redist.14.Latest ' + `
    '--add Microsoft.VisualStudio.ComponentGroup.NativeDesktop.Core ' + `
    '--add Microsoft.VisualStudio.Component.Windows11Sdk.WindowsPerformanceToolkit ' + `
    '--add Microsoft.VisualStudio.Component.CppBuildInsights ' + `
    '--add Microsoft.VisualStudio.ComponentGroup.WebToolsExtensions.CMake ' + `
    '--add Microsoft.VisualStudio.Component.VC.CMake.Project ' + `
    '--add Microsoft.VisualStudio.Component.VC.TestAdapterForBoostTest ' + `
    '--add Microsoft.VisualStudio.Component.VC.TestAdapterForGoogleTest ' + `
    '--add Microsoft.VisualStudio.Component.VC.ASAN ' + `
    '--add Microsoft.VisualStudio.Component.Vcpkg ' + `
    '--add Microsoft.VisualStudio.Component.VC.CLI.Support ' + `
    '--add Microsoft.VisualStudio.Component.VC.Llvm.ClangToolset ' + `
    '--add Microsoft.VisualStudio.Component.VC.Llvm.Clang ' + `
    '--add Microsoft.VisualStudio.ComponentGroup.NativeDesktop.Llvm.Clang ' + `
    '--add Microsoft.VisualStudio.Component.Windows11SDK.26100 ' + `
    '--add Microsoft.VisualStudio.Component.Windows11SDK.22000 ' + `
    '--add Microsoft.Component.NetFX.Native ' + `
    '--add Microsoft.VisualStudio.Component.Graphics ' + `
    '--add Microsoft.VisualStudio.ComponentGroup.UWP.Xamarin ' + `
    '--add Microsoft.VisualStudio.ComponentGroup.UWP.Support ' + `
    '--add Microsoft.VisualStudio.Component.VC.Tools.ARM64EC ' + `
    '--add Microsoft.VisualStudio.Component.UWP.VC.ARM64EC ' + `
    '--add Microsoft.VisualStudio.Component.VC.Tools.ARM64 ' + `
    '--add Microsoft.VisualStudio.Component.UWP.VC.ARM64 ' + `
    '--add Microsoft.VisualStudio.Component.VC.Tools.ARM ' + `
    '--add Microsoft.VisualStudio.ComponentGroup.UWP.VC ' + `
    '--add Microsoft.VisualStudio.Workload.NativeDesktop ' + `
    '--add Microsoft.VisualStudio.Component.WindowsAppSdkSupport.CSharp ' + `
    '--add Microsoft.VisualStudio.ComponentGroup.WindowsAppDevelopment.Prerequisites ' + `
    '--add Microsoft.VisualStudio.ComponentGroup.UWP.NetCoreAndStandard ' + `
    '--add Microsoft.VisualStudio.Workload.Universal ' + `
    '--add Microsoft.VisualStudio.Component.VC.ATL.ARM ' + `
    '--add Microsoft.VisualStudio.Component.VC.ATL.ARM64 ' + `
    '--add Microsoft.VisualStudio.Component.VC.MFC.ARM ' + `
    '--add Microsoft.Net.Component.4.6.1.TargetingPack ' + `
    '--add Microsoft.VisualStudio.Component.VC.14.42.17.12.ARM ' + `
    '--add Microsoft.VisualStudio.Component.VC.14.42.17.12.ARM64 ' + `
    '--add Microsoft.VisualStudio.Component.VC.14.42.17.12.x86.x64 ' + `
    '--add Microsoft.VisualStudio.Component.VC.14.42.17.12.CLI.Support '

$Sku = 'Community'

$ChannelUri = $null

if ($env:install_vs2022_preview) {
	Write-Host "Installing from 'Preview' channel"
	$VSBootstrapperURL = 'https://aka.ms/vs/17/pre/vs_community.exe'
} else {
	Write-Host "Installing from 'Release' channel"
	$VSBootstrapperURL = 'https://aka.ms/vs/17/release/vs_community.exe'

	# This is how to know channelUri for previous versions of VS 2022
	# - Download previous bootstrapper for Professional edition: https://docs.microsoft.com/en-us/visualstudio/releases/2022/history#release-dates-and-build-numbers
	# - Run `.\vs_Professional.exe --layout .\VSLayout
	# - In the output log look for the first line with `/channel`, for example:
	#
	#      Download of 'https://aka.ms/vs/16/release/149189645_1152370582/channel' succeeded using engine 'WebClient'
	# https://aka.ms/vs/16/release/149189645_1152370582/channel is the url to `VisualStudio.16.Release.chman` file.

	# Pin VS 2019 16.5.5 for now because of issues with devenv.com: https://developercommunity.visualstudio.com/content/problem/1048804/cannot-build-project-with-devenvcom-in-visual-stud.html
	#$ChannelUri = 'https://aka.ms/vs/16/release/149189645_1152370582/channel'
	
	#$VSBootstrapperURL = 'https://download.visualstudio.microsoft.com/download/pr/68d6b204-9df0-4fcc-abcc-08ee0eff9cb2/b029547488a9383b0c8d8a9c813e246feb3ec19e0fe55020d4878fde5f0983fe/vs_Community.exe'
}

$ErrorActionPreference = 'Stop'

$VSBootstrapperURL = 'https://aka.ms/vs/17/release/vs_enterprise.exe'
$Sku = 'Enterprise'
$vsPath = "${env:ProgramFiles}\Microsoft Visual Studio\2022\Enterprise"
if (-not (Test-Path $vsPath)) {
    $VSBootstrapperURL = 'https://aka.ms/vs/17/release/vs_professional.exe'
    $Sku = 'Professional'
    $vsPath = "${env:ProgramFiles}\Microsoft Visual Studio\2022\Professional"
    if (-not (Test-Path $vsPath)) {
        $VSBootstrapperURL = 'https://aka.ms/vs/17/release/vs_community.exe'
        $Sku = 'Community'
        $vsPath = "${env:ProgramFiles}\Microsoft Visual Studio\2022\Community"
        if (-not (Test-Path $vsPath)) {
            $vsPath = "${env:ProgramFiles}\Microsoft Visual Studio\2022\Preview"
        }
    }
}

# Install VS
$exitCode = InstallVS -WorkLoads $WorkLoads -Sku $Sku -VSBootstrapperURL $VSBootstrapperURL -ChannelUri $ChannelUri


#Write-Host "Disabling VS-related services"
#if (get-Service SQLWriterw -ErrorAction Ignore) {
#  Stop-Service SQLWriter
#  Set-Service SQLWriter -StartupType Manual
#}
#if (get-Service IpOverUsbSvc -ErrorAction Ignore) {
#  Stop-Service IpOverUsbSvc
#  Set-Service IpOverUsbSvc -StartupType Manual
#}

#Write-Host "Adding Visual Studio 2022 current MSBuild to PATH..." -ForegroundColor Cyan

#Add-Path "$vsPath\MSBuild\Current\Bin"
#Add-Path "$vsPath\Common7\IDE\Extensions\Microsoft\SQLDB\DAC"