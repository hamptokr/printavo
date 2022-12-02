use crate::params;
use crate::Printavo;

#[derive(serde::Deserialize)]
pub struct Order {
    pub id: u32,
    pub order_total: f32,
}

/// Handler for Printavo's Orders API
///
/// Created with [`Printavo::orders`].
pub struct OrdersHandler<'p> {
    printavo: &'p Printavo,
}

impl<'p> OrdersHandler<'p> {
    pub(crate) fn new(printavo: &'p Printavo) -> Self {
        Self { printavo }
    }

    pub fn list(&self) -> ListOrdersBuilder<'_, '_> {
        ListOrdersBuilder::new(self)
    }

    pub fn search(&self) -> SearchOrdersBuilder<'_, '_> {
        SearchOrdersBuilder::new(self)
    }

    pub fn add_payment(
        &self,
        id: u32,
        amount: f32,
        formatted_transaction_date: impl Into<String>,
    ) -> AddPaymentToOrderBuilder<'_, '_> {
        AddPaymentToOrderBuilder::new(self, id, amount, formatted_transaction_date.into())
    }
}

#[derive(serde::Serialize)]
pub struct ListOrdersBuilder<'p, 'b> {
    #[serde(skip)]
    handler: &'b OrdersHandler<'p>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    per_page: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    direction: Option<params::Direction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::iso8601::option")]
    in_production_after: Option<time::OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::iso8601::option")]
    in_production_before: Option<time::OffsetDateTime>,
}

impl<'p, 'b> ListOrdersBuilder<'p, 'b> {
    pub(crate) fn new(handler: &'b OrdersHandler<'p>) -> Self {
        Self {
            handler,
            page: None,
            per_page: None,
            sort_column: None,
            direction: None,
            in_production_after: None,
            in_production_before: None,
        }
    }

    pub fn page(mut self, page: impl Into<u32>) -> Self {
        self.page = Some(page.into());
        self
    }

    pub fn per_page(mut self, per_page: impl Into<u8>) -> Self {
        self.per_page = Some(per_page.into());
        self
    }

    pub fn sort_column(mut self, sort_column: impl Into<String>) -> Self {
        self.sort_column = Some(sort_column.into());
        self
    }

    pub fn direction(mut self, direction: impl Into<params::Direction>) -> Self {
        self.direction = Some(direction.into());
        self
    }

    pub fn in_production_after(
        mut self,
        in_production_after: impl Into<time::OffsetDateTime>,
    ) -> Self {
        self.in_production_after = Some(in_production_after.into());
        self
    }

    pub fn in_production_before(
        mut self,
        in_production_before: impl Into<time::OffsetDateTime>,
    ) -> Self {
        self.in_production_before = Some(in_production_before.into());
        self
    }

    pub async fn send(self) -> crate::Result<crate::Page<Order>> {
        let url = format!("api/{}/orders", self.handler.printavo.version);
        self.handler.printavo.get(url, Some(&self)).await
    }
}

#[derive(serde::Serialize)]
pub struct SearchOrdersBuilder<'p, 'b> {
    #[serde(skip)]
    handler: &'b OrdersHandler<'p>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    per_page: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<String>,
}

impl<'p, 'b> SearchOrdersBuilder<'p, 'b> {
    pub(crate) fn new(handler: &'b OrdersHandler<'p>) -> Self {
        Self {
            handler,
            page: None,
            per_page: None,
            query: None,
        }
    }

    pub fn page(mut self, page: impl Into<u32>) -> Self {
        self.page = Some(page.into());
        self
    }

    pub fn per_page(mut self, per_page: impl Into<u8>) -> Self {
        self.per_page = Some(per_page.into());
        self
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub async fn send(self) -> crate::Result<crate::Page<Order>> {
        let url = format!("api/{}/orders/search", self.handler.printavo.version);
        self.handler.printavo.get(url, Some(&self)).await
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Payment {
    pub id: u32,
    pub order_id: u32,
    #[serde(with = "time::serde::iso8601")]
    pub transaction_date: time::OffsetDateTime,
    pub name: Option<String>,
    pub amount: f32,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: time::OffsetDateTime,
}

#[derive(serde::Serialize)]
pub struct AddPaymentToOrderBuilder<'p, 'b> {
    #[serde(skip)]
    handler: &'b OrdersHandler<'p>,
    #[serde(skip)]
    id: u32,
    book: AddPaymentBook,
}

impl<'p, 'b> AddPaymentToOrderBuilder<'p, 'b> {
    pub(crate) fn new(
        handler: &'b OrdersHandler<'p>,
        id: u32,
        amount: f32,
        formatted_transaction_date: String,
    ) -> Self {
        Self {
            handler,
            id,
            book: AddPaymentBook::new(amount, formatted_transaction_date),
        }
    }

    pub fn book_category_id(mut self, book_category_id: impl Into<u32>) -> Self {
        self.book.book_category_id = Some(book_category_id.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.book.name = Some(name.into());
        self
    }

    pub fn user_generated(mut self, user_generated: impl Into<bool>) -> Self {
        self.book.user_generated = Some(user_generated.into());
        self
    }

    pub async fn send(self) -> crate::Result<Payment> {
        let url = format!(
            "api/{}/orders/{}/add_payment",
            self.handler.printavo.version, self.id
        );
        self.handler.printavo.post(url, Some(&self)).await
    }
}

#[derive(serde::Serialize)]
pub struct AddPaymentBook {
    amount: f32,
    formatted_transaction_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    book_category_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_generated: Option<bool>,
}

impl AddPaymentBook {
    pub fn new(amount: f32, formatted_transaction_date: String) -> Self {
        Self {
            amount,
            formatted_transaction_date,
            book_category_id: None,
            name: None,
            user_generated: None,
        }
    }
}
