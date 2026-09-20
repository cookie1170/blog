let lastHash;

let reloadId = setInterval(reloadIfNeeded, 50);

async function fetchHash() {
  let splitUrl = document.URL.split("/");
  let hashUrl = splitUrl[splitUrl.length - 1] + "/hash.txt";
  let hash = await (await fetch(hashUrl)).text();
  return hash;
}

function reloadIfNeeded() {
  fetchHash()
    .then((hash) => {
      if (!lastHash) {
        lastHash = hash;
        console.log(`got hash '${hash}'`);
        return;
      }
      if (hash !== lastHash) {
        console.log(`hashes differ! '${hash}' != '${lastHash}'`);
        location.reload();
      }
    })
    .catch(() => clearInterval(reloadId));
}
