from abc import ABC, abstractmethod
from typing import Any, Optional


class CompetitionParser(ABC):
    """
    Base class for competition HTML parsers. Subclasses should implement all methods for a given event format.
    """

    @abstractmethod
    def extract_participants(self, html: str, filename: Optional[str] = None) -> Any:
        pass

    @abstractmethod
    def extract_judges(self, html: str, filename: Optional[str] = None) -> Any:
        pass

    @abstractmethod
    def extract_committee(self, html: str) -> Any:
        pass

    @abstractmethod
    def extract_scores(self, html: str, filename: Optional[str] = None) -> Any:
        pass
