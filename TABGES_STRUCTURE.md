# tabges.html Structure and Interpretation

This document provides a detailed breakdown of the headers and data structures found in TopTurnier `tabges.html` files. These files summarize the entire competition's marks and ranks in a "Total Table" (Wertungstabelle Gesamt).

## Grid Logic: Bib-Anchored Columns

The table is fundamentally anchored by **Bib numbers** (Startnummern) in the columns. Most rows provide data for each Bib number in that column.

## Actual Table Headers (First Column Labels)

These labels appear in the first column (`<TD class="td1">` or `<TD class="td3">`) and describe the content of that row for all participants.

### Metadata and Setup

| Actual Header | Meaning |
| :--- | :--- |
| **Anzahl Teilnehmer: [N]** / **No. of participants: [N]** | Indicates the total number of participants. The actual count follows the colon. |
| **Startnummer** / **Start number** | A repeat of the Bib numbers to make the table readable when scrolling. |
| **Wertungsrichter** / **Adjudicators** | Header for the block defining which judges are active. |
| **Class** | (WDSF) Indicates the performance class of the couple (e.g., S, A). |

### Judge Identification Rows

Rows that list multiple names separated by `<br>` are the "Judge Identification" rows.
- **DTV Example:** `AE) Daniel Bumhoffer<br>AW) Monika Gräf<br>AZ) Martin Heilbut...`
- **WDSF Example:** `A) Michelle Abildtrup<br>B) Roberto Albanese<br>C) Monique De Maesschalck-Dom...`

**Explanation:** The letter prefix (e.g., `AE)`) corresponds to the mark given in the cells below. Each `<br>`-separated line matches one component of the multi-line result cells.

### Round Result Rows (DTV-Native)

| Actual Header | Explanation |
| :--- | :--- |
| **Ergebnis der Vorrunde** | Sum of crosses (X) obtained in the opening round. |
| **Ergebnis der 1. Zwischenrunde** | Sum of crosses obtained in the first intermediate round. |
| **Addition Endrunde** | The sum of ranks or skating points from the final. |
| **Qualifiziert für die [Round]** | Qualification status. Cells contain `X` if the couple advanced. |

### Round Result Rows (WDSF)

| Actual Header | Explanation |
| :--- | :--- |
| **Result of 1st round** | Sum of components or crosses for the first round. |
| **Result of final** | The total score or sum of ranks for the final round. |
| **Qualified for [Round]** | Qualification status. Cells contain `X` if the couple advanced. |

### Final Placement Section

| Actual Header | Explanation |
| :--- | :--- |
| **Platz von<br>Platz bis** / **Rank from<br>Rank to** | The final placement range. If a couple is alone in a rank, the second line is often `&nbsp;`. |
| **Aufstiegspunkte** | (DTV) Promotion points awarded to the couple based on the number of defeated opponents. |

### Component/Dance Rows (WDSF Final)

| Actual Header | Explanation |
| :--- | :--- |
| **Samba<br>Cha Cha<br>Rumba<br>Paso Doble<br>Jive** | Labels for multi-line cells containing scores for each specific dance. |
| **Waltz<br>Tango<br>V. Waltz<br>Slowfox<br>Quickstep** | Component labels for Standard competitions. |

## Scoring Cell Interpretation

The data cells (typically `<TD class="td5c">` or `<TD class="td5cv">`) contain the actual marks.

1.  **Multi-line numeric (DTV Intermediate):**
    ```html
    2<br>4<br>1<br>4...
    ```
    Each number represents the number of crosses given by the corresponding judge in the judge block above.
2.  **Consolidated numeric (DTV Final):**
    ```
    21112
    ```
    Each digit represents the rank given by Judge 1, Judge 2, etc., for a specific dance.
3.  **Decimal scores (WDSF):**
    ```html
    35.20<br>34.90...
    ```
    Represent absolute judging scores for each dance component.

## Commonalities vs Differences

### Common among all formats
- **Bib-Anchored:** Everything is indexed by the `Startnummer`.
- **Vertical Orientation:** The first column always defines the "schema" of the rows.
- **Multilingual Support:** TopTurnier uses fixed mapping for German and English based on the event's organization (DTV vs WDSF).

### Differences
- **Numeric Precision:** DTV uses whole numbers for ranks and counts; WDSF uses floats for precise scoring.
- **Rounding Labels:** DTV rounds are usually numbered ("1. Zwischenrunde"), while WDSF rounds are often generic ("2nd round").
- **Final Logic:** DTV shows "Addition Endrunde" (sum of ranks), while WDSF shows "Result of final" (sum of scores).

## Actual Header Rows and Cell Explanations

Below are the raw HTML structures for the table headers and an explanation of each cell.

### DTV Header Structure (German)

```html
<TR>
  <TD class="td2">Anzahl Teilnehmer: 39</TD>
  <TD class="td2c" colspan="15">Startnummer</TD>
</TR>
<TR>
  <TD class="td1" width="25%">Wertungsrichter</TD>
  <TD class="td2gc" width="5%">587<span class="tooltip2gc">Markus Kleander / Carina Jungwirth</span></TD>
  <!-- ... repeated for each Bib number ... -->
</TR>
```

#### Row 1 Explanation:
- **Cell 1 (`td2`):** `Anzahl Teilnehmer: 39`. Displays the total number of registered competitors for this specific event.
- **Cell 2 (`td2c`, `colspan`):** `Startnummer`. A wide header spanning multiple columns that labels the area where Bib numbers are displayed.

#### Row 2 Explanation:
- **Cell 1 (`td1`):** `Wertungsrichter`. In `tabges.html`, this row acts as the primary Bib header. Although labeled "Judges" (Wertungsrichter), the row content is actually the list of competitors.
- **Cell 2+ (`td2gc`):** `587<span class="tooltip2gc">Markus Kleander / Carina Jungwirth</span>`.
  - The visible text (`587`) is the **Bib Number**.
  - The hidden span (`tooltip2gc`) contains the **Full Participant Names** (e.g., "Markus Kleander / Carina Jungwirth"). In a browser, this appears as a tooltip when hovering over the Bib number.

### WDSF Header Structure (English)

```html
<TR>
  <TD class="td2">No. of participants: 93</TD>
  <TD class="td2c" colspan="15">Start number</TD>
</TR>
<TR>
  <TD class="td1" width="25%">Adjudicators</TD>
  <TD class="td2gc" width="5%">108<span class="tooltip2gc">Jean-Pierre Yöndemli / Saskia Maria Skupin</span></TD>
  <!-- ... repeated for each Bib number ... -->
</TR>
```

#### Row 1 Explanation:
- **Cell 1 (`td2`):** `No. of participants: 93`. The English equivalent of the total participant count.
- **Cell 2 (`td2c`, `colspan`):** `Start number`. The English label for the Bib number columns.

#### Row 2 Explanation:
- **Cell 1 (`td1`):** `Adjudicators`. The English equivalent for the Bib header row.
- **Cell 2+ (`td2gc`):** `108<span class="tooltip2gc">...</span>`. Consistent with the DTV format, showing the Bib number with a hidden name tooltip.
