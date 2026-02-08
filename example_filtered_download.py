import os
from dancing_datacollection import load_competition_results

def run_example():
    """
    Demonstrates a filtered download of a single competition from a large event.
    """
    # Target folder for results
    target = "./competition_archive"

    # Example URL of an event index
    # Note: In a real-world scenario, you would use a valid TopTurnier or DTV-Native URL.
    event_url = "https://www.topturnier.de/gsc-ergebnisse/2024/index.htm"

    print(f"Starting filtered download from {event_url}...")
    print("Filtering for: Age Group = Adult, Style = Standard, Level = S")

    try:
        # The orchestrator will:
        # 1. Respect robots.txt.
        # 2. Skip already processed URLs via the manifest in the target folder.
        # 3. Filter for exactly the requested competition criteria.
        # 4. Enforce the Fidelity Gate and Skating System math verification.
        load_competition_results(
            target_folder=target,
            url=event_url,
            age_group="adult",
            style="std",
            level="S",
            download_html=True
        )
        print("\nProcess finished.")
        print(f"Results (if any matches were found and passed validation) are in: {target}")

    except Exception as e:
        # This might fail in the sandbox if there's no internet access or the URL is blocked.
        print(f"\nCould not complete download: {e}")

if __name__ == "__main__":
    if not os.path.exists("./competition_archive"):
        os.makedirs("./competition_archive")
    run_example()
