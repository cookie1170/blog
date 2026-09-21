let lastHash;

let reloadId = setInterval(reloadIfNeeded, 100);

async function fetchHash() {
  let hash = await (await fetch("hash.txt")).text();
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
