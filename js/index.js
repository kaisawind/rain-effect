const rust = import("../pkg/index.js");

rust
  .then(m => {
    let map = {
      dropAlpha: 'img/drop-alpha.png',
      dropColor: 'img/drop-color.png',

      rainFg: 'img/weather/texture-rain-fg.png',
      rainBg: 'img/weather/texture-rain-bg.png',

      stormFg: 'img/weather/texture-storm-lightning-fg.png',
      stormBg: 'img/weather/texture-storm-lightning-bg.png',

      falloutFg: 'img/weather/texture-fallout-fg.png',
      falloutBg: 'img/weather/texture-fallout-bg.png',

      sunFg: 'img/weather/texture-sun-fg.png',
      sunBg: 'img/weather/texture-sun-bg.png',

      drizzleFg: 'img/weather/texture-drizzle-fg.png',
      drizzleBg: 'img/weather/texture-drizzle-bg.png',
    }
    new m.RainEffect("container", map).then((effect) => {
      console.log(effect)
      effect.draw();
    });

  })
  .catch(console.error);
