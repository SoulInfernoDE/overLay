#!/usr/bin/env python3
"""Generates the overLay and Flint logos as SVG and renders PNG sizes with Inkscape.

Wordmarks are converted to outlines (with the font's own kerning), so the SVGs do not
depend on installed fonts. Inter is licensed under the SIL Open Font License 1.1.

Usage:
    python3 tools/branding/build_logos.py --font-dir <dir with Inter-*.ttf> --out assets/branding
"""

import argparse
import shutil
import subprocess
from pathlib import Path

from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont

ICON = 256
PNG_SIZES = (16, 32, 48, 64, 128, 256, 512)

# Palette (matches ui/src/styles/tokens.css)
TEXT_ON_DARK = "#e8ebf0"
TEXT_ON_LIGHT = "#15181e"
TEAL = "#2ed3b7"
TEAL_ON_LIGHT = "#12a58d"
SPARK_ON_LIGHT = "#e8701f"


class Font:
    def __init__(self, path: Path):
        self.font = TTFont(path)
        self.glyphs = self.font.getGlyphSet()
        self.cmap = self.font.getBestCmap()
        self.upm = self.font["head"].unitsPerEm
        self.cap_height = self.font["OS/2"].sCapHeight
        self.x_height = self.font["OS/2"].sxHeight
        self._pair_subtables = self._collect_pair_subtables()

    def _collect_pair_subtables(self):
        if "GPOS" not in self.font:
            return []
        gpos = self.font["GPOS"].table
        indices = sorted(
            {
                i
                for record in gpos.FeatureList.FeatureRecord
                if record.FeatureTag == "kern"
                for i in record.Feature.LookupListIndex
            }
        )
        subtables = []
        for index in indices:
            lookup = gpos.LookupList.Lookup[index]
            for sub in lookup.SubTable:
                if lookup.LookupType == 9:
                    sub = sub.ExtSubTable
                if sub.LookupType == 2:
                    subtables.append(sub)
        return subtables

    def kerning(self, left: str, right: str) -> int:
        for sub in self._pair_subtables:
            if left not in sub.Coverage.glyphs:
                continue
            if sub.Format == 1:
                pair_set = sub.PairSet[sub.Coverage.glyphs.index(left)]
                for record in pair_set.PairValueRecord:
                    if record.SecondGlyph == right:
                        return getattr(record.Value1, "XAdvance", 0) or 0
                continue
            class1 = sub.ClassDef1.classDefs.get(left, 0)
            class2 = sub.ClassDef2.classDefs.get(right, 0)
            value = sub.Class1Record[class1].Class2Record[class2].Value1
            return getattr(value, "XAdvance", 0) or 0
        return 0

    def glyph_name(self, char: str) -> str:
        return self.cmap[ord(char)]

    def outline(self, text: str, size: float, x: float, baseline: float, previous=None):
        """Returns (svg path data, glyph boxes, end x, last glyph name)."""
        scale = size / self.upm
        commands, boxes = [], []
        for char in text:
            name = self.glyph_name(char)
            if previous is not None:
                x += self.kerning(previous, name) * scale
            glyph = self.glyphs[name]
            pen = SVGPathPen(self.glyphs)
            glyph.draw(TransformPen(pen, (scale, 0, 0, -scale, x, baseline)))
            commands.append(pen.getCommands())
            boxes.append((x, glyph.width * scale))
            x += glyph.width * scale
            previous = name
        return " ".join(commands), boxes, x, previous


def sparkle(cx: float, cy: float, r: float) -> str:
    """Four-pointed sparkle with concave sides."""
    return (
        f"M{cx:.1f},{cy - r:.1f} Q{cx:.1f},{cy:.1f} {cx + r:.1f},{cy:.1f} "
        f"Q{cx:.1f},{cy:.1f} {cx:.1f},{cy + r:.1f} Q{cx:.1f},{cy:.1f} {cx - r:.1f},{cy:.1f} "
        f"Q{cx:.1f},{cy:.1f} {cx:.1f},{cy - r:.1f}Z"
    )


def svg(width: float, height: float, body: str, title: str) -> str:
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width:.0f} {height:.0f}" '
        f'width="{width:.0f}" height="{height:.0f}" role="img" aria-label="{title}">\n'
        f"<title>{title}</title>\n{body}\n</svg>\n"
    )


# --- overLay -------------------------------------------------------------------------

def overlay_icon_body(prefix: str = "ol") -> str:
    back = (40, 52, 136, 108)  # x, y, w, h of the "game screen"
    front = (80, 96, 136, 108)  # the overlay pane on top of it
    r = 20
    return f"""<defs>
  <linearGradient id="{prefix}-tile" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0" stop-color="#1f242d"/><stop offset="1" stop-color="#0e1014"/>
  </linearGradient>
  <linearGradient id="{prefix}-pane" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0" stop-color="#6cefd8"/><stop offset="1" stop-color="#22b89e"/>
  </linearGradient>
  <clipPath id="{prefix}-front"><rect x="{front[0]}" y="{front[1]}" width="{front[2]}" height="{front[3]}" rx="{r}"/></clipPath>
</defs>
<rect width="256" height="256" rx="56" fill="url(#{prefix}-tile)"/>
<rect x="1" y="1" width="254" height="254" rx="55" fill="none" stroke="#ffffff" stroke-opacity="0.07" stroke-width="2"/>
<rect x="{back[0]}" y="{back[1]}" width="{back[2]}" height="{back[3]}" rx="{r}" fill="none" stroke="#9aa3b2" stroke-width="14"/>
<rect x="{front[0]}" y="{front[1]}" width="{front[2]}" height="{front[3]}" rx="{r}" fill="url(#{prefix}-pane)"/>
<g clip-path="url(#{prefix}-front)">
  <rect x="{back[0]}" y="{back[1]}" width="{back[2]}" height="{back[3]}" rx="{r}" fill="none" stroke="#062520" stroke-opacity="0.28" stroke-width="14"/>
</g>
<rect x="{front[0] + 76}" y="{front[1] + 62}" width="42" height="10" rx="5" fill="#062520" fill-opacity="0.45"/>
<rect x="{front[0] + 92}" y="{front[1] + 80}" width="26" height="10" rx="5" fill="#062520" fill-opacity="0.45"/>"""


def overlay_logo(font: Font, on_light: bool) -> str:
    size = 150
    baseline = ICON / 2 + font.cap_height * size / font.upm / 2
    x0 = ICON + 44
    first, _, x, prev = font.outline("over", size, x0, baseline)
    second, _, x_end, _ = font.outline("Lay", size, x, baseline, prev)
    text = TEXT_ON_LIGHT if on_light else TEXT_ON_DARK
    accent = TEAL_ON_LIGHT if on_light else TEAL
    body = (
        f"{overlay_icon_body()}\n"
        f'<path d="{first}" fill="{text}"/>\n<path d="{second}" fill="{accent}"/>'
    )
    return svg(x_end + 8, ICON, body, "overLay")


# --- Flint ---------------------------------------------------------------------------

STONE = [(184, 46), (212, 96), (192, 160), (140, 208), (86, 216), (50, 188), (58, 132), (102, 82)]
RIDGE = [(150, 112), (116, 162)]


def flint_icon_body(prefix: str = "fl") -> str:
    p = STONE
    r1, r2 = RIDGE
    facets = [
        ([p[0], p[1], r1], "#8b95a7"),
        ([p[1], p[2], r1], "#647084"),
        ([r1, p[2], p[3], r2], "#4c5567"),
        ([r2, p[3], p[4]], "#3a4251"),
        ([p[4], p[5], r2], "#525b6c"),
        ([p[5], p[6], r2], "#687284"),
        ([p[6], p[7], r1, r2], "#7a8497"),
        ([p[7], p[0], r1], "#9ba4b5"),
    ]
    polys = "\n".join(
        f'<polygon points="{" ".join(f"{x},{y}" for x, y in pts)}" fill="{color}" '
        f'stroke="#1a1d24" stroke-width="3" stroke-linejoin="round"/>'
        for pts, color in facets
    )
    return f"""<defs>
  <linearGradient id="{prefix}-tile" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0" stop-color="#2a2320"/><stop offset="1" stop-color="#131011"/>
  </linearGradient>
  <radialGradient id="{prefix}-glow">
    <stop offset="0" stop-color="#ffb347" stop-opacity="0.55"/><stop offset="1" stop-color="#ff6a2b" stop-opacity="0"/>
  </radialGradient>
  <radialGradient id="{prefix}-spark">
    <stop offset="0" stop-color="#fff6d6"/><stop offset="0.45" stop-color="#ffc35a"/><stop offset="1" stop-color="#ff6a2b"/>
  </radialGradient>
</defs>
<rect width="256" height="256" rx="56" fill="url(#{prefix}-tile)"/>
<rect x="1" y="1" width="254" height="254" rx="55" fill="none" stroke="#ffffff" stroke-opacity="0.07" stroke-width="2"/>
{polys}
<polygon points="{" ".join(f"{x},{y}" for x, y in p)}" fill="none" stroke="#1a1d24" stroke-width="6" stroke-linejoin="round"/>
<circle cx="84" cy="70" r="58" fill="url(#{prefix}-glow)"/>
<path d="{sparkle(84, 70, 40)}" fill="url(#{prefix}-spark)"/>
<path d="{sparkle(48, 112, 13)}" fill="#ffb347"/>
<path d="{sparkle(132, 30, 10)}" fill="#ffc35a"/>"""


def flint_logo(font: Font, on_light: bool) -> str:
    size = 160
    baseline = ICON / 2 + font.cap_height * size / font.upm / 2
    x0 = ICON + 44
    text = TEXT_ON_LIGHT if on_light else TEXT_ON_DARK
    spark = SPARK_ON_LIGHT if on_light else "#ffb347"
    # Dotless i, so a spark can take the place of the dot.
    head, _, x, prev = font.outline("Fl", size, x0, baseline)
    i_path, i_boxes, x, prev = font.outline("ı", size, x, baseline, prev)
    tail, _, x_end, _ = font.outline("nt", size, x, baseline, prev)
    i_x, i_w = i_boxes[0]
    dot_y = baseline - font.cap_height * size / font.upm * 0.93
    body = (
        f"{flint_icon_body()}\n"
        f'<path d="{head} {i_path} {tail}" fill="{text}"/>\n'
        f'<path d="{sparkle(i_x + i_w / 2, dot_y, 24)}" fill="{spark}"/>'
    )
    return svg(x_end + 8, ICON, body, "Flint")


# --- Output --------------------------------------------------------------------------

def render_pngs(icon_svg: Path, out_dir: Path) -> None:
    inkscape = shutil.which("inkscape")
    if not inkscape:
        print("inkscape not found, skipping PNG export")
        return
    for size in PNG_SIZES:
        subprocess.run(
            [inkscape, str(icon_svg), "--export-type=png",
             f"--export-filename={out_dir / f'icon-{size}.png'}", "-w", str(size), "-h", str(size)],
            check=True,
            capture_output=True,
        )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--font-dir", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()

    bold = Font(args.font_dir / "Inter-Bold.ttf")
    extra_bold = Font(args.font_dir / "Inter-ExtraBold.ttf")

    targets = {
        "overlay": ("overLay", overlay_icon_body, lambda light: overlay_logo(bold, light)),
        "flint": ("Flint", flint_icon_body, lambda light: flint_logo(extra_bold, light)),
    }
    for folder, (title, icon_body, logo) in targets.items():
        out = args.out / folder
        out.mkdir(parents=True, exist_ok=True)
        icon_path = out / "icon.svg"
        icon_path.write_text(svg(ICON, ICON, icon_body(), title))
        (out / "logo-dark.svg").write_text(logo(False))
        (out / "logo-light.svg").write_text(logo(True))
        render_pngs(icon_path, out)
        print(f"wrote {out}")


if __name__ == "__main__":
    main()
