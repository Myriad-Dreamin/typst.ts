
import { execSync } from "child_process";

// GITHUB_REF_NAME
const refName = (process.argv[2] || process.env.GITHUB_REF_NAME).trim();

const respJson = execSync(
    `curl -s -H "Accept: application/vnd.github.v3+json" https://api.github.com/repos/Myriad-Dreamin/typst.ts/releases/tags/${refName}`
).toString();

const resp = JSON.parse(respJson || "{}");

const releaseId = resp?.id;
if (!releaseId) {
    console.error(`Release ID not found for tag ${refName}: ${respJson}`);
} else {
    process.stdout.write(`--gh-release-id ${releaseId}`);
}
