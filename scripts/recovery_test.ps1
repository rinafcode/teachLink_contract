param(
  [string]$HealthUrl = 'http://localhost:8080/health',
  [string]$IndexerUrl = 'http://localhost:8081/health',
  [string]$Report = "backups/recovery_reports/$(Get-Date -Format yyyy-MM-ddTHHmmssZ).dr_report.json"
)

New-Item -ItemType Directory -Force -Path (Split-Path $Report) | Out-Null

$start = (Get-Date).ToUniversalTime().ToString('o')

$healthCode = try { (Invoke-WebRequest -Uri $HealthUrl -UseBasicParsing -Method Head).StatusCode.Value__ } catch { 0 }
$indexerCode = try { (Invoke-WebRequest -Uri $IndexerUrl -UseBasicParsing -Method Head).StatusCode.Value__ } catch { 0 }

$pass = $true
if ($healthCode -ne 200 -or $indexerCode -ne 200) { $pass = $false }

$end = (Get-Date).ToUniversalTime().ToString('o')

$reportObj = @{
  start = $start
  end = $end
  health_status_code = $healthCode
  indexer_status_code = $indexerCode
  pass = $pass
} | ConvertTo-Json -Depth 4

Set-Content -Path $Report -Value $reportObj -Encoding UTF8

Write-Output "DR test complete. Report: $Report"
if (-not $pass) { exit 2 }
exit 0
