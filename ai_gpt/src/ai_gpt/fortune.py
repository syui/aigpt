"""AI Fortune system for daily personality variations"""

import json
import random
from datetime import date, datetime, timedelta
from pathlib import Path
from typing import Optional
import logging

from .models import AIFortune


class FortuneSystem:
    """Manages daily AI fortune affecting personality"""
    
    def __init__(self, data_dir: Path):
        self.data_dir = data_dir
        self.fortune_file = data_dir / "fortunes.json"
        self.fortunes: dict[str, AIFortune] = {}
        self.logger = logging.getLogger(__name__)
        self._load_fortunes()
    
    def _load_fortunes(self):
        """Load fortune history from storage"""
        if self.fortune_file.exists():
            with open(self.fortune_file, 'r', encoding='utf-8') as f:
                data = json.load(f)
                for date_str, fortune_data in data.items():
                    # Convert date string back to date object
                    fortune_data['date'] = datetime.fromisoformat(fortune_data['date']).date()
                    self.fortunes[date_str] = AIFortune(**fortune_data)
    
    def _save_fortunes(self):
        """Save fortune history to storage"""
        data = {}
        for date_str, fortune in self.fortunes.items():
            fortune_dict = fortune.model_dump(mode='json')
            fortune_dict['date'] = fortune.date.isoformat()
            data[date_str] = fortune_dict
        
        with open(self.fortune_file, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2)
    
    def get_today_fortune(self) -> AIFortune:
        """Get or generate today's fortune"""
        today = date.today()
        today_str = today.isoformat()
        
        if today_str in self.fortunes:
            return self.fortunes[today_str]
        
        # Generate new fortune
        fortune_value = random.randint(1, 10)
        
        # Check yesterday's fortune for consecutive tracking
        yesterday = (today - timedelta(days=1))
        yesterday_str = yesterday.isoformat()
        
        consecutive_good = 0
        consecutive_bad = 0
        breakthrough_triggered = False
        
        if yesterday_str in self.fortunes:
            yesterday_fortune = self.fortunes[yesterday_str]
            
            if fortune_value >= 7:  # Good fortune
                if yesterday_fortune.fortune_value >= 7:
                    consecutive_good = yesterday_fortune.consecutive_good + 1
                else:
                    consecutive_good = 1
            elif fortune_value <= 3:  # Bad fortune
                if yesterday_fortune.fortune_value <= 3:
                    consecutive_bad = yesterday_fortune.consecutive_bad + 1
                else:
                    consecutive_bad = 1
            
            # Check breakthrough conditions
            if consecutive_good >= 3:
                breakthrough_triggered = True
                self.logger.info("Breakthrough! 3 consecutive good fortunes!")
                fortune_value = 10  # Max fortune on breakthrough
            elif consecutive_bad >= 3:
                breakthrough_triggered = True
                self.logger.info("Breakthrough! 3 consecutive bad fortunes!")
                fortune_value = random.randint(7, 10)  # Good fortune after bad streak
        
        fortune = AIFortune(
            date=today,
            fortune_value=fortune_value,
            consecutive_good=consecutive_good,
            consecutive_bad=consecutive_bad,
            breakthrough_triggered=breakthrough_triggered
        )
        
        self.fortunes[today_str] = fortune
        self._save_fortunes()
        
        self.logger.info(f"Today's fortune: {fortune_value}/10")
        return fortune
    
    def get_personality_modifier(self, fortune: AIFortune) -> dict[str, float]:
        """Get personality modifiers based on fortune"""
        base_modifier = fortune.fortune_value / 10.0
        
        modifiers = {
            "optimism": base_modifier,
            "energy": base_modifier * 0.8,
            "patience": 1.0 - (abs(5.5 - fortune.fortune_value) * 0.1),
            "creativity": 0.5 + (base_modifier * 0.5),
            "empathy": 0.7 + (base_modifier * 0.3)
        }
        
        # Breakthrough effects
        if fortune.breakthrough_triggered:
            modifiers["confidence"] = 1.0
            modifiers["spontaneity"] = 0.9
        
        return modifiers