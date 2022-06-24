const rust = import("../pkg/index.js");

rust
  .then(m => {
    m.load_image('img/drop-alpha.png').then((image) => {
      console.log(image)
    });
  })
  .catch(console.error);
