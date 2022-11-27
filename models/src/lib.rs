use std::str::FromStr;

use regex::Regex;

#[derive(Debug, Clone, Copy)]
pub enum Category {
    FoodAndDrinks,
    OperationalSpends,
    Rent,
    UkrReponsibilities,
}

impl ToString for Category {
    fn to_string(&self) -> String {
        let string_literal = match self {
            Category::FoodAndDrinks => "Поживні речовини",
            Category::OperationalSpends => "Операційні витрати",
            Category::Rent => "Аренда",
            Category::UkrReponsibilities => "Українські зобовʼязання",
        };
        string_literal.to_owned()
    }
}

impl FromStr for Category {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Поживні речовини" => Ok(Category::FoodAndDrinks),
            "Операційні витрати" => Ok(Category::OperationalSpends),
            "Аренда" => Ok(Category::Rent),
            "Українські зобовʼязання" => Ok(Category::UkrReponsibilities),
            _ => Err(()),
        }
    }
}

impl Category {
    pub fn to_storage_id(&self) -> &str {
        match self {
            Category::FoodAndDrinks => "de702077-821a-4b59-9ab0-949ca954c4a6",
            Category::OperationalSpends => "3e392471-bd4a-4cde-befb-e52e8f31b26d",
            Category::Rent => "278a679f-cde4-44aa-be5a-2876b67d080b",
            Category::UkrReponsibilities => "M?[|",
        }
    }
}

#[derive(Debug)]
pub struct Check {
    pub entries: Vec<CheckEntry>,
    pub spent: f64,
}

#[derive(Debug)]
pub struct CheckEntry(String, f64);

impl FromStr for Check {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut check = Check {
            entries: s
                .lines()
                .map(|line| CheckEntry::from_str(line).expect("Line is corrupted"))
                .collect::<Vec<CheckEntry>>(),
            spent: 0.0,
        };
        check.spent = check.entries.iter().map(|entry| entry.1).sum::<f64>();
        Ok(check)
    }
}

impl FromStr for CheckEntry {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let content = s
            .split_ascii_whitespace()
            .into_iter()
            .collect::<Vec<&str>>();
        let re = Regex::new(r"[-+]?\d*\.\d+|\d+").unwrap();
        let mat = re.find(content.last().unwrap());
        if let Some(matched) = mat {
            let price = matched.as_str().parse::<f64>().unwrap();
            let description = content
                .iter()
                .take(content.len() - 1)
                .map(|el| el.to_string())
                .collect::<Vec<String>>()
                .join(" ");
            return Ok(CheckEntry(description, price));
        }
        Err(())
    }
}

impl ToString for CheckEntry {
    fn to_string(&self) -> String {
        format!("{} — ціна: {} (в євро)", self.0, self.1)
    }
}

pub struct RemainAmount {
    pub category: Category,
    pub amount: f64,
}

pub trait ToSchedule {
    fn replenish_money_schedules(&self) -> Vec<String>;
}

impl ToSchedule for Category {
    fn replenish_money_schedules(&self) -> Vec<String> {
        match self {
            // https://crontab.cronhub.io/
            Category::FoodAndDrinks | Category::OperationalSpends => vec![
                String::from_str("0 0 0 15 * *").unwrap(),
                String::from_str("0 0 0 15 * *").unwrap(),
            ],
            Category::Rent | Category::UkrReponsibilities => {
                vec![String::from_str("0 0 0 1 * *").unwrap()]
            }
        }
    }
}

pub trait ToMoneyReplenishment {
    fn to_replenishment_amount(&self) -> f64;
}

impl ToMoneyReplenishment for Category {
    fn to_replenishment_amount(&self) -> f64 {
        match self {
            Category::FoodAndDrinks => 155.0,
            Category::OperationalSpends => 250.0,
            Category::UkrReponsibilities => 310.0,
            Category::Rent => 1140.0,
        }
    }
}
