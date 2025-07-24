#[cfg(feature = "runtime-benchmarks")]
mod benchmarking {
    use super::*;
    use frame_benchmarking::{benchmarks, account, vec};
    use frame_system::RawOrigin;

    const SEED: u32 = 0;
    
    fn create_community_bench_helper<T: Config>(
        creator: T::AccountId,
        name: Vec<u8>,
        description: Vec<u8>,
        community_type: CommunityType,
        icon: Vec<u8>,
        social_user_name: Vec<u8>,
        proposal_reason: Vec<u8>,
    ) -> Result<u32, &'static str> {
        let community_id = Pallet::<T>::next_community_id().map_err(|_| "Failed to get community ID")?;
        Pallet::<T>::create_community(
            RawOrigin::Signed(creator.clone()).into(),
            name,
            description,
            community_type,
            icon,
            social_user_name,
            proposal_reason,
        )?;
        Ok(community_id)
    }

    benchmarks! {
        create_community {
            let creator: T::AccountId = account("creator", 0, SEED);
            let name = vec![b'k'; T::MaxNameLength::get() as usize];
            let description = vec![b'd'; T::MaxDescLength::get() as usize];
            let icon = vec![b'i'; T::MaxIconLength::get() as usize];
            let social_user_name = vec![b's'; T::MaxSocialUserNameLength::get() as usize];
            let proposal_reason = vec![b'r'; T::MaxReasonLength::get() as usize];
            let community_type = CommunityType::Public { monthly_fee: 10 };
        }: _(RawOrigin::Signed(creator.clone()), name, description, community_type, icon, social_user_name, proposal_reason)
        verify {
            assert!(Communities::<T>::get(1).is_some());
        }

        submit_vote {
            let creator: T::AccountId = account("creator", 0, SEED);
            let voter: T::AccountId = account("voter", 1, SEED);
            let name = vec![b'k'; T::MaxNameLength::get() as usize];
            let description = vec![b'd'; T::MaxDescLength::get() as usize];
            let icon = vec![b'i'; T::MaxIconLength::get() as usize];
            let social_user_name = vec![b's'; T::MaxSocialUserNameLength::get() as usize];
            let proposal_reason = vec![b'r'; T::MaxReasonLength::get() as usize];
            let community_type = CommunityType::Public { monthly_fee: 0 };

            let community_id = create_community_bench_helper::<T>(creator, name, description, community_type, icon, social_user_name, proposal_reason)?;
        }: _(RawOrigin::Signed(voter.clone()), community_id, true, vec![b'v'; T::MaxSocialUserNameLength::get() as usize])
        verify {
            assert!(CommunityCreateVoters::<T>::get(community_id).iter().any(|v| v.user == voter));
        }

        add_member {
            let creator: T::AccountId = account("creator", 0, SEED);
            let user: T::AccountId = account("user", 1, SEED);
            let name = vec![b'k'; T::MaxNameLength::get() as usize];
            let description = vec![b'd'; T::MaxDescLength::get() as usize];
            let icon = vec![b'i'; T::MaxIconLength::get() as usize];
            let social_user_name = vec![b's'; T::MaxSocialUserNameLength::get() as usize];
            let proposal_reason = vec![b'r'; T::MaxReasonLength::get() as usize];
            let community_type = CommunityType::Public { monthly_fee: 0 };

            let community_id = create_community_bench_helper::<T>(creator, name, description, community_type, icon, social_user_name, proposal_reason)?;
        }: _(RawOrigin::Signed(user.clone()), community_id, vec![b'm'; T::MaxSocialUserNameLength::get() as usize])
        verify {
            assert!(CommunityMembers::<T>::get(community_id).iter().any(|m| m.user == user));
        }

        remove_member {
            let creator: T::AccountId = account("creator", 0, SEED);
            let user: T::AccountId = account("user", 1, SEED);
            let name = vec![b'k'; T::MaxNameLength::get() as usize];
            let description = vec![b'd'; T::MaxDescLength::get() as usize];
            let icon = vec![b'i'; T::MaxIconLength::get() as usize];
            let social_user_name = vec![b's'; T::MaxSocialUserNameLength::get() as usize];
            let proposal_reason = vec![b'r'; T::MaxReasonLength::get() as usize];
            let community_type = CommunityType::Public { monthly_fee: 0 };

            let community_id = create_community_bench_helper::<T>(creator.clone(), name, description, community_type, icon, social_user_name, proposal_reason)?;
            Pallet::<T>::add_member(RawOrigin::Signed(user.clone()).into(), community_id, vec![b'm'; T::MaxSocialUserNameLength::get() as usize])?;
        }: _(RawOrigin::Signed(creator.clone()), community_id, user.clone(), vec![b'm'; T::MaxSocialUserNameLength::get() as usize])
        verify {
            assert!(!CommunityMembers::<T>::get(community_id).iter().any(|m| m.user == user));
        }

        delete_community {
            let creator: T::AccountId = account("creator", 0, SEED);
            let name = vec![b'k'; T::MaxNameLength::get() as usize];
            let description = vec![b'd'; T::MaxDescLength::get() as usize];
            let icon = vec![b'i'; T::MaxIconLength::get() as usize];
            let social_user_name = vec![b's'; T::MaxSocialUserNameLength::get() as usize];
            let proposal_reason = vec![b'r'; T::MaxReasonLength::get() as usize];
            let community_type = CommunityType::Public { monthly_fee: 0 };
            let community_id = create_community_bench_helper::<T>(creator.clone(), name, description, community_type, icon, social_user_name, proposal_reason)?;
        }: _(RawOrigin::Signed(creator.clone()), community_id)
        verify {
            assert!(Communities::<T>::get(community_id).is_none());
        }
    }
}
