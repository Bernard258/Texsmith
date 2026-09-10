# Elsevier manuscript

Uses `elsarticle` in single-column `preprint` mode with front matter, affiliations, keywords, and the `elsarticle-num` numbered bibliography style. Follow the target journal instructions for review spacing, citation style, and declarations.

Source: [elsarticle package and documentation](https://ctan.org/pkg/elsarticle).

Edit the authors and abstract in `main.tex`, the paper text in
`sections/body.tex`, and citations in `references.bib`. Replace the example
citation and illustrative table. Add images to `figures/`.

Compile from the Reports workspace with `texsmith build REPORT-NAME`.
Tectonic runs BibTeX and the required LaTeX passes automatically. The class
and bibliography style are loaded from its TeX bundle on demand; no separate
class download is needed. These are editable starters, not submission guarantees.
