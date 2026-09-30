use oxidauth_kernel::user_authorities::{
    UserAuthorityWithAuthority,
    find_user_authority_by_user_id_and_authority_id::FindUserAuthorityByUserIdAndAuthorityId,
};

pub type FindUserAuthorityByUserIdAndAuthorityIdReq = FindUserAuthorityByUserIdAndAuthorityId;

pub type FindUserAuthorityByUserIdAndAuthorityIdRes = UserAuthorityWithAuthority;
