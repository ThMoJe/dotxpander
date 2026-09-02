# PowerShell script to commit, update tag v0.2.0, and push to GitHub
# Triggers the GitHub Actions 'Build and Release' workflow.

$ErrorActionPreference = "Stop"

$tagName = "v0.2.0"
$commitMsg = "Build 0.2.0 release again"

Write-Host "`n[1/4] Staging changes..." -ForegroundColor Cyan
git add -A

Write-Host "[2/4] Committing with message: '$commitMsg'..." -ForegroundColor Cyan
git commit -m $commitMsg

Write-Host "[3/4] Updating tag $tagName to new commit..." -ForegroundColor Cyan
git tag -f $tagName

Write-Host "[4/4] Pushing branch and tag to GitHub..." -ForegroundColor Cyan
git push origin main
git push origin $tagName --force

Write-Host "`nRelease push complete! GitHub Actions build has been triggered." -ForegroundColor Green
