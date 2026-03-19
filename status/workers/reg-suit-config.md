Created a draft visual CI config and workflow under `status/`.

The workflow intentionally references `asset_catalogue` and `scripts/visual-regression.sh` because those entry points were named by the tier-3 visual task, but they are not present in this repository slice and creating them would have exceeded the allowed `status/` scope.
