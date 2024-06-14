use pgml_components::Component;
use std::collections::HashMap;

pub use crate::components::{self, cms::index_link::IndexLink, NavLink, StaticNav, StaticNavLink};
use crate::{Notification, NotificationLevel};
use components::notifications::marketing::{AlertBanner, FeatureBanner};
use components::notifications::product::ProductBanner;

use crate::models::Cluster;
use sailfish::TemplateOnce;
use sqlx::postgres::types::PgMoney;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::{Column, Executor, PgPool, Row, Statement, TypeInfo, ValueRef};

use crate::models;
use crate::utils::tabs;

pub mod docs;

use crate::components::layouts::Head;

#[derive(TemplateOnce, Default)]
#[template(path = "content/not_found.html")]
pub struct NotFound {}

#[derive(TemplateOnce, Default)]
#[template(path = "content/error.html")]
pub struct Error {
    pub error: String,
}

#[derive(TemplateOnce, Clone, Default)]
#[template(path = "layout/base.html")]
pub struct Layout {
    pub head: Head,
    pub content: Option<String>,
    pub user: Option<models::User>,
    pub toc_links: Vec<docs::TocLink>,
    pub footer: Option<String>,
    pub alert_banner: AlertBanner,
    pub feature_banner: FeatureBanner,
}

impl Layout {
    pub fn new(title: &str, context: Option<&crate::guards::Cluster>) -> Self {
        let (head, footer, user) = match context.as_ref() {
            Some(context) => (
                Head::new().title(title).context(&context.context.head_items),
                Some(context.context.marketing_footer.clone()),
                Some(context.context.user.clone()),
            ),
            None => (Head::new().title(title), None, None),
        };

        Layout {
            head,
            footer,
            user,
            alert_banner: AlertBanner::from_notification(Notification::next_alert(context)),
            feature_banner: FeatureBanner::from_notification(Notification::next_feature(context)),
            ..Default::default()
        }
    }

    pub fn description(&mut self, description: &str) -> &mut Self {
        self.head.description = Some(description.to_owned());
        self
    }

    pub fn image(&mut self, image: &str) -> &mut Self {
        self.head.image = Some(image.to_owned());
        self
    }

    pub fn canonical(&mut self, canonical: &str) -> &mut Self {
        self.head.canonical = Some(canonical.to_owned());
        self
    }

    pub fn content(&mut self, content: &str) -> &mut Self {
        self.content = Some(content.to_owned());
        self
    }

    pub fn user(&mut self, user: &models::User) -> &mut Self {
        self.user = Some(user.to_owned());
        self
    }

    pub fn toc_links(&mut self, toc_links: &[docs::TocLink]) -> &mut Self {
        self.toc_links = toc_links.to_vec();
        self
    }

    pub fn render<T>(&mut self, template: T) -> String
    where
        T: sailfish::TemplateOnce,
    {
        self.content = Some(template.render_once().unwrap());
        (*self).clone().into()
    }

    pub fn footer(&mut self, footer: String) -> &mut Self {
        self.footer = Some(footer);
        self
    }
}

impl From<Layout> for String {
    fn from(layout: Layout) -> String {
        layout.render_once().unwrap()
    }
}

#[derive(TemplateOnce, Clone, Default)]
#[template(path = "layout/web_app_base.html")]
pub struct WebAppBase<'a> {
    pub content: Option<String>,
    pub breadcrumbs: Vec<NavLink<'a>>,
    pub head: Head,
    pub dropdown_nav: StaticNav,
    pub product_left_nav: StaticNav,
    pub body_components: Vec<Component>,
    pub cluster: Cluster,
    pub product_banners_high: Vec<ProductBanner>,
    pub product_banner_medium: ProductBanner,
}

impl<'a> WebAppBase<'a> {
    pub fn new(title: &str, context: &crate::guards::Cluster) -> Self {
        let head = Head::new().title(title).context(&context.context.head_items);
        let cluster = context.context.cluster.clone();

        let all_product_high_level = context
            .notifications
            .clone()
            .unwrap_or_else(|| vec![])
            .into_iter()
            .filter(|n: &Notification| n.level == NotificationLevel::ProductHigh)
            .enumerate()
            .map(|(i, n)| ProductBanner::from_notification(Some(&n)).set_show_modal_on_load(i == 0))
            .collect::<Vec<ProductBanner>>();

        WebAppBase {
            head,
            cluster,
            dropdown_nav: context.context.dropdown_nav.clone(),
            product_left_nav: context.context.product_left_nav.clone(),
            product_banners_high: all_product_high_level,
            product_banner_medium: ProductBanner::from_notification(Notification::next_product_of_level(
                context,
                NotificationLevel::ProductMedium,
            )),
            ..Default::default()
        }
    }

    pub fn breadcrumbs(&mut self, breadcrumbs: Vec<NavLink<'a>>) -> &mut Self {
        self.breadcrumbs = breadcrumbs.to_owned();
        self
    }

    pub fn disable_upper_nav(&mut self) -> &mut Self {
        let links: Vec<StaticNavLink> = self
            .product_left_nav
            .links
            .iter()
            .map(|item| item.to_owned().disabled(true))
            .collect();
        self.product_left_nav = StaticNav { links };
        self
    }

    pub fn content(&mut self, content: &str) -> &mut Self {
        self.content = Some(content.to_owned());
        self
    }

    pub fn body_components(&mut self, components: Vec<Component>) -> &mut Self {
        self.body_components = components;
        self
    }

    pub fn render<T>(&mut self, template: T) -> String
    where
        T: sailfish::TemplateOnce,
    {
        self.content = Some(template.render_once().unwrap());
        (*self).clone().into()
    }
}

impl<'a> From<WebAppBase<'a>> for String {
    fn from(layout: WebAppBase) -> String {
        layout.render_once().unwrap()
    }
}

#[derive(TemplateOnce)]
#[template(path = "content/article.html")]
pub struct Article {
    pub content: String,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/panels/projects.html")]
pub struct Projects {
    pub projects: Vec<models::Project>,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/tabs/project_tab.html")]
pub struct ProjectTab {
    pub project_id: i64,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/panels/notebooks.html")]
pub struct Notebooks {
    pub notebooks: Vec<models::Notebook>,
    pub new: bool,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/panels/notebook.html")]
pub struct Notebook {
    pub notebook: models::Notebook,
    pub cells: Vec<models::Cell>,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/panels/cell.html")]
pub struct Cell {
    pub notebook: models::Notebook,
    pub cell: models::Cell,
    pub edit: bool,
    pub selected: bool,
}

#[derive(TemplateOnce)]
#[template(path = "content/undo.html")]
pub struct Undo {
    pub notebook: models::Notebook,
    pub cell: models::Cell,
    pub bust_cache: String,
}

#[derive(TemplateOnce, Default)]
#[template(path = "content/sql.html")]
pub struct Sql {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub execution_duration: std::time::Duration,
}

impl Sql {
    pub async fn new(pool: &PgPool, query: &str) -> anyhow::Result<Sql> {
        let prepared_stmt = pool.prepare(query).await?;
        let cols = prepared_stmt.columns();

        let mut columns = Vec::new();
        let mut rows = Vec::new();

        cols.iter().for_each(|c| columns.push(c.name().to_string()));

        let now = std::time::Instant::now();
        let result = prepared_stmt.query().fetch_all(pool).await?;
        let execution_duration = now.elapsed();

        for row in result.iter() {
            let mut values = Vec::new();

            for (i, _) in cols.iter().enumerate() {
                let type_ = cols[i].type_info().name();

                let null_check = row.try_get_raw(i)?;

                if null_check.is_null() {
                    values.push("".to_string());
                    continue;
                }

                let value = match type_ {
                    "TEXT" | "VARCHAR" | "CHAR(N)" | "NAME" => {
                        let value: String = row.try_get(i)?;
                        value
                    }

                    "TEXT[]" | "VARCHAR[]" => {
                        let value: Vec<String> = row.try_get(i)?;
                        format!("{:?}", value)
                    }

                    "INT8" | "BIGINT" | "BIGSERIAL" => {
                        let value: i64 = row.try_get(i)?;
                        value.to_string()
                    }

                    "INT8[]" | "BIGINT[]" => {
                        let value: Vec<i64> = row.try_get(i)?;
                        format!("{:?}", value)
                    }

                    "INT" | "SERIAL" | "INT4" => {
                        let value: i32 = row.try_get(i)?;
                        value.to_string()
                    }

                    "INT[]" | "INT4[]" => {
                        let value: Vec<i32> = row.try_get(i)?;
                        format!("{:?}", value)
                    }

                    "INT2" | "SMALLINT" | "SMALLSERIAL" => {
                        let value: i16 = row.try_get(i)?;
                        value.to_string()
                    }

                    "INT2[]" | "SMALLINT[]" => {
                        let value: Vec<i16> = row.try_get(i)?;
                        format!("{:?}", value)
                    }

                    "DOUBLE PRECISION" | "FLOAT8" => {
                        let value: f64 = row.try_get(i)?;
                        value.to_string()
                    }

                    "DOUBLE PRECISION[]" | "FLOAT8[]" => {
                        let value: Vec<f64> = row.try_get(i)?;
                        format!("{:?}", value)
                    }

                    "FLOAT4" | "REAL" => {
                        let value: f32 = row.try_get(i)?;
                        value.to_string()
                    }

                    "FLOAT4[]" | "REAL[]" => {
                        let value: Vec<f32> = row.try_get(i)?;
                        format!("{:?}", value)
                    }

                    "BYTEA" => {
                        // let value: Vec<u8> = row.try_get(i)?;
                        "<binary>".to_string()
                    }

                    "BOOL" => {
                        let value: bool = row.try_get(i)?;
                        value.to_string()
                    }

                    "NUMERIC" => {
                        let value: sqlx::types::BigDecimal = row.try_get(i)?;
                        value.to_string()
                    }

                    "TIMESTAMP" => {
                        let value: PrimitiveDateTime = row.try_get(i)?;
                        let (hour, minute, second, milli) = value.as_hms_milli();
                        let (year, month, day) = value.to_calendar_date();

                        format!("{}-{}-{} {}:{}:{}.{}", year, month, day, hour, minute, second, milli)
                    }

                    "MONEY" => {
                        let value: PgMoney = row.try_get(i)?;
                        value.to_bigdecimal(2).to_string()
                    }

                    "RECORD" => "OK".to_string(),

                    "JSON" | "JSONB" => {
                        let value: serde_json::Value = row.try_get(i)?;
                        serde_json::to_string(&value)?
                    }

                    "vector" => {
                        let value: pgvector::Vector = row.try_get(i)?;
                        format!("{:?}", value.to_vec())
                    }

                    unknown => {
                        // TODO
                        // Implement everything here: https://docs.rs/sqlx/latest/sqlx/postgres/types/index.html
                        return Err(anyhow::anyhow!("Unsupported type: {}", unknown));
                    }
                };

                values.push(value);
            }

            rows.push(values);
        }

        Ok(Sql {
            columns,
            rows,
            execution_duration,
        })
    }
}

#[derive(TemplateOnce)]
#[template(path = "content/sql_error.html")]
pub struct SqlError {
    pub error: String,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/panels/models.html")]
pub struct Models {
    pub projects: Vec<models::Project>,
    pub models: HashMap<i64, Vec<models::Model>>,
    // pub min_scores: HashMap<i64, f64>,
    // pub max_scores: HashMap<i64, f64>,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/panels/model.html")]
pub struct Model {
    pub model: models::Model,
    pub project: models::Project,
    pub snapshot: Option<models::Snapshot>,
    pub deployed: bool,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/panels/snapshots.html")]
pub struct Snapshots {
    pub snapshots: Vec<models::Snapshot>,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/panels/snapshot.html")]
pub struct Snapshot {
    pub snapshot: models::Snapshot,
    pub models: Vec<models::Model>,
    pub projects: HashMap<i64, models::Project>,
    pub samples: HashMap<String, Vec<f32>>,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/panels/deployments.html")]
pub struct Deployments {
    pub projects: Vec<models::Project>,
    pub deployments: HashMap<i64, Vec<models::Deployment>>,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/panels/deployment.html")]
pub struct Deployment {
    pub project: models::Project,
    pub model: models::Model,
    pub deployment: models::Deployment,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/panels/project.html")]
pub struct Project {
    pub project: models::Project,
    pub models: Vec<models::Model>,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/panels/uploader.html")]
pub struct Uploader {
    pub error: Option<String>,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/panels/uploaded.html")]
pub struct Uploaded {
    pub sql: Sql,
    pub columns: Vec<String>,
    pub table_name: String,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/dashboard.html")]
pub struct Dashboard<'a> {
    pub tabs: tabs::Tabs<'a>,
}

impl Dashboard<'_> {
    pub fn new<'a>(tabs: tabs::Tabs<'a>) -> Dashboard<'a> {
        Dashboard { tabs }
    }
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/tabs/notebooks_tab.html")]
pub struct NotebooksTab;

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/tabs/notebook_tab.html")]
pub struct NotebookTab {
    pub id: i64,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/tabs/projects_tab.html")]
pub struct ProjectsTab;

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/tabs/deployments_tab.html")]
pub struct DeploymentsTab {
    pub deployment_id: Option<i64>,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/tabs/models_tab.html")]
pub struct ModelsTab;

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/tabs/model_tab.html")]
pub struct ModelTab {
    pub model_id: i64,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/tabs/snapshots_tab.html")]
pub struct SnapshotsTab;

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/tabs/snapshot_tab.html")]
pub struct SnapshotTab {
    pub snapshot_id: i64,
}

#[derive(TemplateOnce)]
#[template(path = "content/dashboard/tabs/uploader_tab.html")]
pub struct UploaderTab {
    pub table_name: Option<String>,
}

#[derive(TemplateOnce)]
#[template(path = "content/playground.html")]
pub struct Playground;
