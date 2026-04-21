module.exports = async ({ github, context, core }) => {
  const { VERSION, FILES } = process.env;
  const owner = context.repo.owner;
  const repo = context.repo.repo;

  const fs = require("fs");
  const path = require("path");

  const isDraft = VERSION.includes("-rc");

  let release;
  for await (const res of github.paginate.iterator(
    github.rest.repos.listReleases,
    { owner, repo, per_page: 100 },
  )) {
    release = res.data.find((r) => r.tag_name === VERSION);
    if (release) break;
  }

  if (!release) {
    core.info(`Release ${VERSION} not found, creating (draft=${isDraft})...`);
    const created = await github.rest.repos.createRelease({
      owner,
      repo,
      tag_name: VERSION,
      name: VERSION,
      generate_release_notes: true,
      draft: isDraft,
    });
    release = created.data;
  }

  const existingAssets = (
    await github.rest.repos.listReleaseAssets({
      owner,
      repo,
      release_id: release.id,
    })
  ).data;

  const files = FILES.split("\n").map((f) => f.trim()).filter(Boolean);
  for (const fileName of files) {
    const duplicate = existingAssets.find((a) => a.name === fileName);
    if (duplicate) {
      await github.rest.repos.deleteReleaseAsset({
        owner,
        repo,
        asset_id: duplicate.id,
      });
    }
    const data = fs.readFileSync(path.join(process.cwd(), fileName));
    const contentType = fileName.endsWith(".txt")
      ? "text/plain"
      : "application/zip";
    await github.rest.repos.uploadReleaseAsset({
      owner,
      repo,
      release_id: release.id,
      name: fileName,
      data,
      headers: {
        "content-type": contentType,
        "content-length": data.length,
      },
    });
    core.info(`Uploaded ${fileName}`);
  }

  core.info(`Release ${VERSION} is ready: ${release.html_url}`);
};
