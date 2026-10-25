# Tests CI : les captures UI, la langue et les caches du runtime sont globaux.
# Les deux lots sont séquentiels pour éviter qu'un test de capture ou de plugin
# ne modifie l'état observé par un autre test en parallèle.
# Les tests en échec sont publiés en annotations (lisibles sans se connecter à GitHub).
$log = "test-output.log"
Remove-Item $log -ErrorAction SilentlyContinue
$failed = $false
foreach ($set in @(@("--", "--skip", "plugin_", "--test-threads=1"), @("plugin_", "--", "--test-threads=1"))) {
    cargo test --locked @set 2>&1 | Tee-Object -FilePath $log -Append
    if ($LASTEXITCODE -ne 0) { $failed = $true }
}
if ($failed) {
    Select-String -Path $log -Pattern "panicked at|^test .* FAILED|^---- |error:" |
        Select-Object -First 25 | ForEach-Object { "::error title=cargo test::$($_.Line.Trim())" }
    exit 1
}
