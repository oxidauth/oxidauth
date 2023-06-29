use oxidauth_repository::authorities::insert_authority::*;

use crate::prelude::*;

#[async_trait]
impl InsertAuthority for Database {
    async fn insert_authority(
        &self,
        params: InsertAuthorityParams,
    ) -> Result<AuthorityRow, InsertAuthorityError> {
        // let result = sqlx::query_as::<_, AuthorityRow>(include_str!("./insert_authority.sql"))
        //     .bind(&params.name)

        todo!()
    }
}
