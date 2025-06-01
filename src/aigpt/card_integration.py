"""ai.card integration module for ai.gpt MCP server"""

from typing import Dict, Any, List, Optional
import httpx
from pathlib import Path
import json
from datetime import datetime
import logging

logger = logging.getLogger(__name__)


class CardIntegration:
    """Integration with ai.card system"""
    
    def __init__(self, api_base_url: str = "http://localhost:8001"):
        self.api_base_url = api_base_url
        self.client = httpx.AsyncClient()
    
    async def get_user_cards(self, did: str) -> List[Dict[str, Any]]:
        """Get cards for a specific user by DID"""
        try:
            response = await self.client.get(
                f"{self.api_base_url}/api/v1/cards/user/{did}"
            )
            if response.status_code == 200:
                return response.json()
            else:
                logger.error(f"Failed to get cards: {response.status_code}")
                return []
        except Exception as e:
            logger.error(f"Error getting user cards: {e}")
            return []
    
    async def draw_card(self, did: str) -> Optional[Dict[str, Any]]:
        """Draw a new card for user (gacha)"""
        try:
            response = await self.client.post(
                f"{self.api_base_url}/api/v1/gacha/draw",
                json={"did": did}
            )
            if response.status_code == 200:
                return response.json()
            else:
                logger.error(f"Failed to draw card: {response.status_code}")
                return None
        except Exception as e:
            logger.error(f"Error drawing card: {e}")
            return None
    
    async def get_card_info(self, card_id: int) -> Optional[Dict[str, Any]]:
        """Get detailed information about a specific card"""
        try:
            response = await self.client.get(
                f"{self.api_base_url}/api/v1/cards/{card_id}"
            )
            if response.status_code == 200:
                return response.json()
            else:
                return None
        except Exception as e:
            logger.error(f"Error getting card info: {e}")
            return None
    
    async def sync_with_atproto(self, did: str) -> bool:
        """Sync card data with atproto"""
        try:
            response = await self.client.post(
                f"{self.api_base_url}/api/v1/sync/atproto",
                json={"did": did}
            )
            return response.status_code == 200
        except Exception as e:
            logger.error(f"Error syncing with atproto: {e}")
            return False
    
    async def close(self):
        """Close the HTTP client"""
        await self.client.aclose()


def register_card_tools(app, card_integration: CardIntegration):
    """Register ai.card tools to FastAPI app"""
    
    @app.get("/get_user_cards", operation_id="get_user_cards")
    async def get_user_cards(did: str) -> List[Dict[str, Any]]:
        """Get all cards owned by a user"""
        cards = await card_integration.get_user_cards(did)
        return cards
    
    @app.post("/draw_card", operation_id="draw_card")
    async def draw_card(did: str) -> Dict[str, Any]:
        """Draw a new card (gacha) for user"""
        result = await card_integration.draw_card(did)
        if result:
            return {
                "success": True,
                "card": result
            }
        else:
            return {
                "success": False,
                "error": "Failed to draw card"
            }
    
    @app.get("/get_card_details", operation_id="get_card_details")
    async def get_card_details(card_id: int) -> Dict[str, Any]:
        """Get detailed information about a card"""
        info = await card_integration.get_card_info(card_id)
        if info:
            return info
        else:
            return {"error": f"Card {card_id} not found"}
    
    @app.post("/sync_cards_atproto", operation_id="sync_cards_atproto")
    async def sync_cards_atproto(did: str) -> Dict[str, str]:
        """Sync user's cards with atproto"""
        success = await card_integration.sync_with_atproto(did)
        if success:
            return {"status": "Cards synced successfully"}
        else:
            return {"status": "Failed to sync cards"}
    
    @app.get("/analyze_card_collection", operation_id="analyze_card_collection")
    async def analyze_card_collection(did: str) -> Dict[str, Any]:
        """Analyze user's card collection"""
        cards = await card_integration.get_user_cards(did)
        
        if not cards:
            return {
                "total_cards": 0,
                "rarity_distribution": {},
                "message": "No cards found"
            }
        
        # Analyze collection
        rarity_count = {}
        total_power = 0
        
        for card in cards:
            rarity = card.get("rarity", "common")
            rarity_count[rarity] = rarity_count.get(rarity, 0) + 1
            total_power += card.get("power", 0)
        
        return {
            "total_cards": len(cards),
            "rarity_distribution": rarity_count,
            "average_power": total_power / len(cards) if cards else 0,
            "strongest_card": max(cards, key=lambda x: x.get("power", 0)) if cards else None
        }