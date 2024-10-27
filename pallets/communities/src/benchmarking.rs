#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
    create_community {
        let caller: T::AccountId = whitelisted_caller();
        let name = b"My Community".to_vec();
        let description = b"Community Description".to_vec();
        let community_type = CommunityType::Public;
        let icon = b"Icon".to_vec();
        let social_user_name = b"SocialName".to_vec();
        let proposal_reason = b"Reason".to_vec();
    }: _(RawOrigin::Signed(caller.clone()), name, description, community_type, icon, social_user_name, proposal_reason)
    verify {
        assert!(Communities::<T>::get(1).is_some());
    }

    add_member {
        let caller: T::AccountId = whitelisted_caller();
        let community_id: u32 = 1;
        let social_user_name = b"MemberSocial".to_vec();

        Pallet::<T>::create_community(
            RawOrigin::Signed(caller.clone()).into(),
            b"My Community".to_vec(),
            b"Description".to_vec(),
            CommunityType::Public,
            b"Icon".to_vec(),
            b"Social".to_vec(),
            b"Reason".to_vec()
        ).expect("Community creation should succeed");

        let member: T::AccountId = account("member", 0, 0);
    }: _(RawOrigin::Signed(member.clone()), community_id, social_user_name)
    verify {
        assert!(CommunityMembers::<T>::get(community_id).iter().any(|m| m.user == member));
    }

    // Adicione mais benchmarks para outras funções, como `approve_member`, `remove_member`, etc.
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
