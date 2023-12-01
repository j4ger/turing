use super::{
    card::{Card, CardColor},
    player::{Player, PlayerID},
};

pub(crate) type Money = u32;

#[derive(Clone, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
#[archive(check_bytes)]
pub(crate) struct GameState {
    pub(crate) deck: Vec<Vec<Card>>,
    pub(crate) money: Vec<Money>,
    pub(crate) players: Vec<Player>,
    pub(crate) stage: GameStage,
    pub(crate) current_round: usize,
    pub(crate) values: [[Money; 5]; 5],
}

#[derive(Clone, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
#[archive(check_bytes)]
pub(crate) enum GameStage {
    WaitingForNextCard(PlayerID),
    WaitingForDoubleTarget {
        double_card: CardPair,
        current: PlayerID,
    },
    WaitingForMarkedPrice {
        starter: PlayerID,
        target: AuctionTarget,
    },
    AuctionInAction {
        state: AuctionState,
        target: AuctionTarget,
    },
}

pub(crate) type CardPair = (PlayerID, Card);
pub(crate) type MoneyPair = (PlayerID, Money);

#[derive(Clone, Copy, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize, PartialEq, Eq)]
#[archive(check_bytes)]
pub(crate) enum AuctionTarget {
    Single(CardPair),
    Double {
        double_card: CardPair,
        target_card: CardPair,
    },
}

#[derive(Clone, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
#[archive(check_bytes)]
pub(crate) enum AuctionState {
    Free {
        host: PlayerID,
        highest: MoneyPair,
        time_end: i64,
        calls: u8,
    },
    Circle {
        starter: PlayerID,
        current_player: PlayerID,
        highest: MoneyPair,
    },
    Fist {
        bids: Vec<Money>,
        can_end: bool,
    },
    Marked {
        starter: PlayerID,
        current: MoneyPair,
    },
}

#[derive(Clone, Copy)]
pub(crate) enum ShouldEnd {
    Yes(CardColor),
    No,
}

impl GameState {
    pub(crate) fn round_should_end(&self) -> ShouldEnd {
        let mut counters = vec![0u32; 5];
        for player in self.players.iter() {
            for card in player.owned_cards.iter() {
                if let Some(count) = counters.get_mut(card.color.index()) {
                    *count += 1;
                }
            }
        }

        if let Some(position) = counters.iter().position(|x| x == &5) {
            let color = CardColor::from_index(position);
            ShouldEnd::Yes(color)
        } else {
            ShouldEnd::No
        }
    }
}

impl GameStage {
    pub(crate) fn is_player_active(&self, player_id: PlayerID) -> bool {
        match &self {
            GameStage::WaitingForNextCard(next) => player_id == *next,
            GameStage::WaitingForDoubleTarget { current, .. } => player_id == *current,
            GameStage::WaitingForMarkedPrice { starter, .. } => player_id == *starter,
            GameStage::AuctionInAction { state, .. } => match state {
                AuctionState::Free { .. } => true,
                AuctionState::Circle { current_player, .. } => player_id == *current_player,
                AuctionState::Fist { .. } => true,
                AuctionState::Marked { current, .. } => player_id == current.0,
            },
        }
    }
}

