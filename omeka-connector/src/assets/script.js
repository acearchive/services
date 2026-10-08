const copyShortFileUrlButton = document.getElementById("copy-short-url-button");
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
