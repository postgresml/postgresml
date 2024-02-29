use pgml_components::component;
use sailfish::TemplateOnce;

struct Perk {
    icon: String,
    title: String,
    info: String,
    color: String,
}

impl Perk {
    pub fn new() -> Perk {
        Perk {
            icon: String::new(),
            title: String::new(),
            info: String::new(),
            color: String::new(),
        }
    }

    pub fn icon(mut self, icon: &str) -> Perk {
        self.icon = icon.to_string();
        self
    }

    pub fn title(mut self, title: &str) -> Perk {
        self.title = title.to_string();
        self
    }

    pub fn info(mut self, info: &str) -> Perk {
        self.info = info.to_string();
        self
    }

    pub fn color(mut self, color: &str) -> Perk {
        self.color = color.to_string();
        self
    }
}

#[derive(TemplateOnce, Default)]
#[template(path = "sections/employment_benefits/template.html")]
pub struct EmploymentBenefits {
    perks: Vec<Perk>,
}

impl EmploymentBenefits {
    pub fn new() -> EmploymentBenefits {
        EmploymentBenefits {
            perks: Vec::from([
                Perk::new().icon("computer").color("blue").title("Remote-first").info("Work from anywhere in the United States."),
                Perk::new().icon("flight_takeoff").color("orange").title("Relocate if you want").info("We’ll offer a relocation package if you’re interested in moving to the beautiful bay area."),
                Perk::new().icon("favorite").color("pink").title("Platinum-tier insurance").info("We cover the max allowable (99%) health, dental and vision premiums for platinum tier insurance plans."),
                Perk::new().icon("payments").color("green").title("Stipends").info("$5k/year hardware budget, $500/month home office reimbursement as well as learning and development/conference stipends."),
                Perk::new().icon("wifi_off").color("purple").title("Unlimited PTO").info("And we strongly encourage you to use it to stay healthy and happy. It’s typical for team members to take 3-4 weeks per year in addition to holidays."),
                Perk::new().icon("group").color("party").title("Connect in person").info("The entire team comes together for quarterly on-sites where we do fun stuff like wine tasting and bowling. If you live in the Bay Area, we hike and hang out every Wednesday."),
              ])
        }
    }
}

component!(EmploymentBenefits);
