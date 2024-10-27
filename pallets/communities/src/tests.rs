#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_noop, assert_ok};
    use sp_core::H256;
    use frame_system::EventRecord;
    use crate::Event;

    fn new_test_ext() -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
        t.into()
    }

    #[test]
    fn create_community_should_work() {
        new_test_ext().execute_with(|| {
            let creator = 1;
            let name = b"My Community".to_vec();
            let description = b"Community Description".to_vec();
            let community_type = CommunityType::Public;
            let icon = b"Icon".to_vec();
            let social_user_name = b"CreatorSocial".to_vec();
            let proposal_reason = b"Proposal Reason".to_vec();

            assert_ok!(Pallet::create_community(
                Origin::signed(creator),
                name.clone(),
                description.clone(),
                community_type,
                icon.clone(),
                social_user_name.clone(),
                proposal_reason.clone()
            ));

            let community = Communities::<Test>::get(1).expect("Community should exist");
            assert_eq!(community.name, name);
            assert_eq!(community.description, description);
            assert_eq!(community.created_by, creator);
        });
    }

    #[test]
    fn add_member_should_work() {
        new_test_ext().execute_with(|| {
            let creator = 1;
            let member = 2;
            let community_id = 1;
            let social_user_name = b"MemberSocial".to_vec();

            // Primeiro, crie uma comunidade
            assert_ok!(Pallet::create_community(
                Origin::signed(creator),
                b"My Community".to_vec(),
                b"Description".to_vec(),
                CommunityType::Public,
                b"Icon".to_vec(),
                b"CreatorSocial".to_vec(),
                b"Proposal Reason".to_vec()
            ));

            // Em seguida, adicione o membro
            assert_ok!(Pallet::add_member(
                Origin::signed(member),
                community_id,
                social_user_name
            ));

            let members = CommunityMembers::<Test>::get(community_id);
            assert!(members.iter().any(|m| m.user == member));
        });
    }

    // Mais testes podem ser adicionados para outras funções do pallet, como:
    // - Teste para `approve_member`
    // - Teste para `remove_member`
    // - Teste para `delete_community`
}
