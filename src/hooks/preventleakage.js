webview.addEventListener('before-navigate', (event) => {
    // Prevent the navigation request from completing
    event.preventDefault();
  });
  