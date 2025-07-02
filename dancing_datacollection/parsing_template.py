from .parsing_base import CompetitionParser

class TemplateParser(CompetitionParser):
    """
    Template for a new competition HTML format parser.
    Implement all extract_* methods for your format.
    """
    def extract_participants(self, html):
        """Extract participants from HTML. Return a list of dicts."""
        # TODO: Implement for your format
        return []

    def extract_judges(self, html):
        """Extract judges from HTML. Return a list of dicts."""
        # TODO: Implement for your format
        return []

    def extract_committee(self, html):
        """Extract committee from HTML. Return a list of dicts."""
        # TODO: Implement for your format
        return []

    def extract_scores(self, html):
        """Extract scores from HTML. Return a list of dicts."""
        # TODO: Implement for your format
        return []

    def extract_final_scoring(self, html):
        """Extract final scoring from HTML. Return a list of dicts."""
        # TODO: Implement for your format
        return [] 