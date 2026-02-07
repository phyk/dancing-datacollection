from dancing_datacollection import load_competition_results
import os

def test_import():
    print("Successfully imported load_competition_results")

if __name__ == "__main__":
    test_import()
    # Caution: running this without a valid URL or local server will likely fail in the download step.
    # But we can at least verify the call signature.
    try:
        load_competition_results("./results", "http://localhost:8080", download_html=True)
    except Exception as e:
        print(f"Caught expected error (as no server is running): {e}")
