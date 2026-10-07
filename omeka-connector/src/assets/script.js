const copyShortFileUrlButton = document.getElementById("copy-short-url-button");
const copyShortRawFileUrlButton = document.getElementById(
  "copy-short-raw-url-button",
);
const toast = document.getElementById("toast");

copyShortFileUrlButton.addEventListener("click", () => {
  const url = copyShortFileUrlButton.dataset.url;
  navigator.clipboard.writeText(url).then(() => {
    toast.hidden = false;
    setTimeout(() => {
      toast.hidden = true;
    }, 1500);
  });
});

copyShortRawFileUrlButton.addEventListener("click", () => {
  const url = copyShortRawFileUrlButton.dataset.url;
  navigator.clipboard.writeText(url).then(() => {
    toast.hidden = false;
    setTimeout(() => {
      toast.hidden = true;
    }, 1500);
  });
});
