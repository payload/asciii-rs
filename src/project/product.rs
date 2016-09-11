use currency::Currency;
use bill::{BillProduct, Tax};

use util::yaml;
use util::yaml::Yaml;

use super::{ProductResult,ProductError};
use super::spec::to_currency;

//#[derive(Debug)] // manually implemented
#[derive(Copy,Clone)]
pub struct Product<'a> {
    pub name: &'a str,
    pub unit: Option<&'a str>,
    pub tax: Tax,
    pub price: Currency
}


impl<'a> Product<'a>{
    pub fn from_old_format<'y>(name: &'y str, values: &'y Yaml) -> ProductResult<Product<'y>> {
        let default_tax = ::CONFIG.get_f64("defaults/tax")
            .expect("Faulty config: field defaults/tax does not contain a value");
        Ok(Product {
            name: name,
            unit: yaml::get_str(values, "unit"),
            price: try!(yaml::get_f64(values, "price")
                .ok_or(ProductError::InvalidPrice)
                .map(to_currency)),
            tax: yaml::get_f64(values, "tax").unwrap_or(default_tax).into(),
        })
    }

    pub fn from_new_format(desc: &Yaml) -> ProductResult<Product> {
        let default_tax = ::CONFIG.get_f64("defaults/tax")
            .expect("Faulty config: field defaults/tax does not contain a value");
        Ok(Product {
            name: yaml::get_str(desc, "name").unwrap_or("unnamed"),
            unit: yaml::get_str(desc, "unit"),
            price: try!(yaml::get_f64(desc, "price")
                .ok_or(ProductError::InvalidPrice)
                .map(to_currency)),
            tax: yaml::get_f64(desc, "tax").unwrap_or(default_tax).into(),
        })
    }

    pub fn from_desc_and_value<'y>(desc: &'y Yaml, values: &'y Yaml) -> ProductResult<Product<'y>> {
        match *desc {
            yaml::Yaml::String(ref name) => Self::from_old_format(name, values),
            yaml::Yaml::Hash(_) => Self::from_new_format(desc),
            _ => Err(ProductError::UnknownFormat),
        }
    }
}

impl<'a> BillProduct for Product<'a>{
    fn price(&self) -> Currency {self.price}
    fn name(&self) -> String {self.name.to_owned()}
    fn tax(&self) -> Tax {self.tax}
}
