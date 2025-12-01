export const Monet = {
  hexToRgb: (hex) => {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? { r: parseInt(result[1], 16), g: parseInt(result[2], 16), b: parseInt(result[3], 16) } : null;
  },
  rgbToHsl: (r, g, b) => {
    r /= 255; g /= 255; b /= 255;
    const max = Math.max(r, g, b), min = Math.min(r, g, b);
    let h, s, l = (max + min) / 2;
    if (max === min) h = s = 0;
    else {
      const d = max - min;
      s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
      switch (max) {
        case r: h = (g - b) / d + (g < b ? 6 : 0); break;
        case g: h = (b - r) / d + 2; break;
        case b: h = (r - g) / d + 4; break;
      }
      h /= 6;
    }
    return { h: h * 360, s: s * 100, l: l * 100 };
  },
  hslToHex: (h, s, l) => {
    l /= 100;
    const a = s * Math.min(l, 1 - l) / 100;
    const f = n => {
      const k = (n + h / 30) % 12;
      const color = l - a * Math.max(Math.min(k - 3, 9 - k, 1), -1);
      return Math.round(255 * color).toString(16).padStart(2, '0');
    };
    return `#${f(0)}${f(8)}${f(4)}`;
  },
  getTone: (baseHsl, lAdjust, sAdjust = 0, hAdjust = 0) => {
    return Monet.hslToHex((baseHsl.h + hAdjust) % 360, Math.max(0, Math.min(100, baseHsl.s + sAdjust)), lAdjust);
  },
  apply: (seedHex, isDark) => {
    if (!seedHex) return;
    const rgb = Monet.hexToRgb(seedHex);
    if (!rgb) return;
    const base = Monet.rgbToHsl(rgb.r, rgb.g, rgb.b);
    const p = { h: base.h, s: Math.min(base.s, 90) };
    const s = { h: base.h, s: Math.min(base.s, 40) };
    const t = { h: (base.h + 60) % 360, s: Math.min(base.s, 50) };
    const n = { h: base.h, s: Math.min(base.s, 10) };
    const nv = { h: base.h, s: Math.min(base.s, 15) };
    const err = { h: 350, s: 80 };
    const tones = isDark ? {
      primary: Monet.getTone(p, 80), onPrimary: Monet.getTone(p, 20),
      primaryCont: Monet.getTone(p, 30), onPrimaryCont: Monet.getTone(p, 90),
      secondary: Monet.getTone(s, 80), onSecondary: Monet.getTone(s, 20),
      secondaryCont: Monet.getTone(s, 30), onSecondaryCont: Monet.getTone(s, 90),
      tertiary: Monet.getTone(t, 80), onTertiary: Monet.getTone(t, 20),
      tertiaryCont: Monet.getTone(t, 30), onTertiaryCont: Monet.getTone(t, 90),
      error: Monet.getTone(err, 80), onError: Monet.getTone(err, 20),
      errorCont: Monet.getTone(err, 30), onErrorCont: Monet.getTone(err, 90),
      bg: Monet.getTone(n, 6), onBg: Monet.getTone(n, 90),
      surf: Monet.getTone(n, 6), onSurf: Monet.getTone(n, 90),
      surfVar: Monet.getTone(nv, 30), onSurfVar: Monet.getTone(nv, 80),
      outline: Monet.getTone(nv, 60), outlineVar: Monet.getTone(nv, 30),
      surfContLow: Monet.getTone(n, 10), surfCont: Monet.getTone(n, 12),
      surfContHigh: Monet.getTone(n, 17), surfContHighest: Monet.getTone(n, 22),
    } : {
      primary: Monet.getTone(p, 40), onPrimary: Monet.getTone(p, 100),
      primaryCont: Monet.getTone(p, 90), onPrimaryCont: Monet.getTone(p, 10),
      secondary: Monet.getTone(s, 40), onSecondary: Monet.getTone(s, 100),
      secondaryCont: Monet.getTone(s, 90), onSecondaryCont: Monet.getTone(s, 10),
      tertiary: Monet.getTone(t, 40), onTertiary: Monet.getTone(t, 100),
      tertiaryCont: Monet.getTone(t, 90), onTertiaryCont: Monet.getTone(t, 10),
      error: Monet.getTone(err, 40), onError: Monet.getTone(err, 100),
      errorCont: Monet.getTone(err, 90), onErrorCont: Monet.getTone(err, 10),
      bg: Monet.getTone(n, 99), onBg: Monet.getTone(n, 10),
      surf: Monet.getTone(n, 99), onSurf: Monet.getTone(n, 10),
      surfVar: Monet.getTone(nv, 90), onSurfVar: Monet.getTone(nv, 30),
      outline: Monet.getTone(nv, 50), outlineVar: Monet.getTone(nv, 80),
      surfContLow: Monet.getTone(n, 96), surfCont: Monet.getTone(n, 94),
      surfContHigh: Monet.getTone(n, 92), surfContHighest: Monet.getTone(n, 90),
    };
    const root = document.documentElement.style;
    for (const [key, value] of Object.entries(tones)) {
      let cssVar = '';
      if(key === 'bg') cssVar = '--md-sys-color-background';
      else if(key === 'onBg') cssVar = '--md-sys-color-on-background';
      else if(key === 'surf') cssVar = '--md-sys-color-surface';
      else if(key === 'onSurf') cssVar = '--md-sys-color-on-surface';
      else if(key === 'surfVar') cssVar = '--md-sys-color-surface-variant';
      else if(key === 'onSurfVar') cssVar = '--md-sys-color-on-surface-variant';
      else if(key === 'primary') cssVar = '--md-sys-color-primary';
      else if(key === 'onPrimary') cssVar = '--md-sys-color-on-primary';
      else if(key === 'primaryCont') cssVar = '--md-sys-color-primary-container';
      else if(key === 'onPrimaryCont') cssVar = '--md-sys-color-on-primary-container';
      else if(key === 'secondary') cssVar = '--md-sys-color-secondary';
      else if(key === 'onSecondary') cssVar = '--md-sys-color-on-secondary';
      else if(key === 'secondaryCont') cssVar = '--md-sys-color-secondary-container';
      else if(key === 'onSecondaryCont') cssVar = '--md-sys-color-on-secondary-container';
      else if(key === 'tertiary') cssVar = '--md-sys-color-tertiary';
      else if(key === 'onTertiary') cssVar = '--md-sys-color-on-tertiary';
      else if(key === 'tertiaryCont') cssVar = '--md-sys-color-tertiary-container';
      else if(key === 'onTertiaryCont') cssVar = '--md-sys-color-on-tertiary-container';
      else if(key === 'error') cssVar = '--md-sys-color-error';
      else if(key === 'onError') cssVar = '--md-sys-color-on-error';
      else if(key === 'errorCont') cssVar = '--md-sys-color-error-container';
      else if(key === 'onErrorCont') cssVar = '--md-sys-color-on-error-container';
      else if(key === 'outline') cssVar = '--md-sys-color-outline';
      else if(key === 'outlineVar') cssVar = '--md-sys-color-outline-variant';
      else if(key === 'surfContLow') cssVar = '--md-sys-color-surface-container-low';
      else if(key === 'surfCont') cssVar = '--md-sys-color-surface-container';
      else if(key === 'surfContHigh') cssVar = '--md-sys-color-surface-container-high';
      else if(key === 'surfContHighest') cssVar = '--md-sys-color-surface-container-highest';
      if (cssVar) root.setProperty(cssVar, value);
    }
  }
};