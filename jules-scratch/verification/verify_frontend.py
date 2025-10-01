import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect

from dancing_datacollection.data_defs.dances import Dance
from dancing_datacollection.data_defs.participant import Participant
from dancing_datacollection.data_defs.results import (
    ResultRound,
    FinalRoundPlacing,
    DanceScore,
)
from dancing_datacollection.html_generate import generate_erg_html

# 1. Create sample data
participants = [
    Participant(
        name_one="John Doe",
        name_two="Jane Doe",
        number=1,
        club="Dance Club 1",
        ranks=[1],
    ),
    Participant(
        name_one="Peter Pan",
        name_two="Wendy Darling",
        number=2,
        club="Dance Club 2",
        ranks=[2],
    ),
]

placing1 = FinalRoundPlacing(
    rank="1",
    participant=participants[0],
    dance_scores={
        "SlowWaltz": DanceScore(marks="11111", place=1.0),
        "Tango": DanceScore(marks="11111", place=1.0),
    },
    total_score=2.0,
)

placing2 = FinalRoundPlacing(
    rank="2",
    participant=participants[1],
    dance_scores={
        "SlowWaltz": DanceScore(marks="22222", place=2.0),
        "Tango": DanceScore(marks="22222", place=2.0),
    },
    total_score=4.0,
)

final_round = ResultRound(
    name="Final Round",
    placings=[placing1, placing2],
)

# 2. Generate HTML
html_content = generate_erg_html([final_round], title="Test Competition")

# 3. Write HTML to a temporary file
verification_dir = Path("jules-scratch/verification")
html_file_path = verification_dir / "test_page.html"
with open(html_file_path, "w", encoding="utf-8") as f:
    f.write(html_content)

# 4. Use Playwright to take a screenshot
with sync_playwright() as p:
    browser = p.chromium.launch()
    page = browser.new_page()

    # Use an absolute file path
    absolute_path = os.path.abspath(html_file_path)
    page.goto(f"file://{absolute_path}")

    expect(page.locator("text=Test Competition")).to_be_visible()

    screenshot_path = verification_dir / "verification.png"
    page.screenshot(path=screenshot_path)

    browser.close()

print(f"Screenshot saved to {screenshot_path}")