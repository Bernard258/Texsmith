# IEEE conference

Uses `IEEEtran` in `conference` mode with US Letter paper, two columns, IEEE author blocks, index terms, and the `IEEEtran` BibTeX style. Change `letterpaper` to `a4paper` if your conference requires it. Do not add `geometry` or override margins and fonts. Follow the conference instructions for anonymity, page limits, funding statements, and any required copyright notice.

Sources: [IEEE conference authoring tools](https://conferences.ieeeauthorcenter.ieee.org/write-your-paper/authoring-tools-and-templates/), [IEEEtran documentation](https://www.michaelshell.org/tex/ieeetran/).

Edit the authors and abstract in `main.tex`, the paper text in
`sections/body.tex`, and citations in `references.bib`. Replace the example
citation and illustrative table. Add images to `figures/`.

Compile from the Reports workspace with `texsmith build REPORT-NAME`.
Tectonic runs BibTeX and the required LaTeX passes automatically. The class
and bibliography style are loaded from its TeX bundle on demand; no separate
class download is needed. These are editable starters, not submission guarantees.

The template loads T1 font encoding before the class so Tectonic uses the
Times fonts selected by IEEEtran rather than falling back to Latin Modern.
