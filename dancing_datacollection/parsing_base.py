from abc import ABC, abstractmethod


class CompetitionParser(ABC):
    """
    Base class for competition HTML parsers. Subclasses should implement all methods for a given event format.
    """

    @abstractmethod
    def extract_participants(self, html):
        pass

    @abstractmethod
    def extract_judges(self, html):
        pass

    @abstractmethod
    def extract_committee(self, html):
        pass

    @abstractmethod
    def extract_scores(self, html):
        pass
