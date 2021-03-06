use rustc_serialize::json::{ToJson, Json};
use chrono::*;
use bill::{Bill, BillItem, ItemList, Tax};
use ordered_float::OrderedFloat;

use std::process;
use std::error::Error;

use super::Project;
use super::product::Product;
use super::spec;
use util::currency_to_string;
use storage::Storable;

fn opt_to_json<T: ::std::fmt::Display>(opt:Option<T>) -> Json{
    match opt{
        Some(t) => Json::String(t.to_string()),
        None    => Json::Null
    }
}

fn s(s:&str) -> String { String::from(s) }

fn itemlist_to_json(tax:&Tax, list: &ItemList<Product>) -> Json {
    let gross_sum = list.gross_sum();
    let tax_sum  = list.tax_sum();
    let map = btreemap!{
        s("tax_value")   => (tax.into_inner()*100.0).to_json(),
        s("gross_sum")   => currency_to_string(&gross_sum).to_json(),
        s("tax_sum") => currency_to_string(&tax_sum).to_json(),
        s("has_tax")  => (tax.into_inner() > 0f64).to_json()
    };
    map.to_json()
}

fn taxes_by_tax_to_json(bill: &Bill<Product>) -> Json {
    bill.iter()
        .map(|(tax, list)| { itemlist_to_json(tax, list) })
        .rev()
        .collect::<Vec<Json>>()
        .to_json()
}

impl ToJson for Project{
    fn to_json(&self) -> Json{
        use ::project::spec::*;

        let s = |s:&str| String::from(s);

        let opt_str = |opt:Option<&str>| opt.map(|e|e.to_owned()).to_json() ;
        let y = &self.yaml;
        let dmy = |date:Option<Date<UTC>>| date.map(|d|d.format("%d.%m.%Y").to_string()).to_json();

        let item_to_json = |item:&BillItem<Product>, tax:OrderedFloat<f64>| btreemap!{
            s("name") => item.product.name.to_json(),
            s("price") => currency_to_string(&item.product.price).to_json(),
            s("unit") => item.product.unit.unwrap_or_else(||"").to_json(),
            s("amount") => item.amount.to_json(),
            s("cost") => currency_to_string(&item.gross()).to_json(),
            s("tax") => tax.into_inner().to_json()
        }.to_json();

        let bill_to_json = |bill:&Bill<Product>| bill.as_items_with_tax()
                                                     .into_iter()
                                                     .map(|(tax, item)| item_to_json(item,tax) )
                                                     .collect::<Vec<Json>>()
                                                     .to_json();


        let (offer,invoice) = match self.bills() {
            Ok(bills) => bills,
            Err(err) => {
                error!("Cannot create Bill {}", err.description());
                process::exit(1);
            },
        };

        let map = btreemap!{
            //String::from("adressing") => ,

            s("bills") =>  btreemap!{
                s("offer") => bill_to_json(&offer),
                s("invoice") => bill_to_json(&invoice),
            }.to_json(),

            s("client") => btreemap!{
                s("email")      => opt_str(client::email(y)),
                s("last_name")  => opt_str(client::last_name(y)),
                s("first_name") => opt_str(client::first_name(y)),
                s("full_name")  => client::full_name(y).to_json(),
                s("title")      => opt_str(client::title(y)),
                s("address")    => opt_str(client::address(y)),
                s("addressing") => client::addressing(y).to_json(),
            }.to_json(),


            s("event") => btreemap!{
                s("name")    => self.name().to_json(),
                s("date")    => dmy(project::date(y)),
                s("manager") => self.manager().to_json(),
            }.to_json(),


            s("offer") => btreemap!{
                s("number") => offer::number(y).to_json(),
                s("date")   => dmy(spec::date::offer(y)),
                s("sums")   => taxes_by_tax_to_json(&offer),
                s("net_total")  => currency_to_string(&offer.net_total()).to_json(),
                s("gross_total")  => currency_to_string(&offer.gross_total()).to_json(),
            }.to_json(),

            s("invoice") => btreemap!{
                s("date")   => dmy(spec::date::invoice(y)),
                s("number")      => invoice::number_str(y).to_json(),
                s("number_long") => invoice::number_long_str(y).to_json(),
                s("official") => invoice::official(y).to_json(),
                s("sums")   => taxes_by_tax_to_json(&invoice),
                s("net_total")  => currency_to_string(&invoice.net_total()).to_json(),
                s("gross_total")  => currency_to_string(&invoice.gross_total()).to_json(),
            }.to_json(),

            s("hours") => btreemap!{
                s("time")   => opt_to_json(hours::total(y)),
                s("salary") => opt_to_json(hours::salary(y).map(|ref c|currency_to_string(c)))
            }.to_json(),

        };
        Json::Object(map)
    }
}

impl<'a> ToJson for Product<'a> {
    fn to_json(&self) -> Json {
        let s = |s: &str| String::from(s);
        Json::Object(btreemap!{
            s("name")     => self.name.to_json(),
            s("unit")     => self.unit.map(|s|s.to_owned()).to_json(),
            s("tax")      => self.tax.to_string().to_json(),
            s("price")    => currency_to_string(&self.price).to_json(),
            s("currency") => self.price.0.map(|s|s.to_string()).to_json(),
        })
    }
}

