/// PubMed Format tags.
///
/// See https://pubmed.ncbi.nlm.nih.gov/help/#pubmed-format
#[non_exhaustive]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum PubmedTag {
    /// AB - Abstract: English language abstract taken directly from the published article
    Abstract,
    /// AD - Affiliation: Author or corporate author addresses
    Affiliation,
    /// AID - Article Identifier: Article ID values supplied by the publisher may include
    /// the pii (controlled publisher identifier), doi (digital object identifier), or
    /// book accession
    ArticleIdentifier,
    /// AU - Author: Authors
    Author,
    /// AUID - Author Identifier: Unique identifier associated with an author, corporate
    /// author, or investigator name
    AuthorIdentifier,
    /// BTI - Book Title: Book Title
    BookTitle,
    /// \[...\] and so on...
    CopyrightInformation,
    CommentIn,
    CorporateAuthor,
    ConflictOfInterest,
    CommentOn,
    Chapter,
    CreateDate,
    CorrectedAndRepublishedFrom,
    CorrectedAndRepublishedIn,
    ContributionDate,
    CollectionTitle,
    CompletionDate,
    DatasetDescribedIn,
    DatasetUseReportedIn,
    DateOfElectronicPublication,
    PublicationDate,
    DateRevised,
    ExpressionOfConcernFor,
    ExpressionOfConcernIn,
    EntryDate,
    ErratumFor,
    ErratumIn,
    Editor,
    Edition,
    FullAuthorName,
    FullEditorName,
    FullInvestigatorName,
    FullPersonalNameAsSubject,
    GeneralNote,
    GrantsAndFunding,
    GeneSymbol,
    Issue,
    Investigator,
    InvestigatorAffiliation,
    ISSN,
    ISBN,
    NLMUniqueID,
    FullJournalTitle,
    Language,
    LocationID,
    ModificationDate,
    MeSHTerms,
    MeSHDate,
    ManuscriptIdentifier,
    SubstanceName,
    OtherAbstract,
    OtherAbstractLanguage,
    OtherCopyrightInformation,
    OtherID,
    OriginalReportIn,
    OtherTerm,
    OtherTermOwner,
    Owner,
    Publisher,
    Pagination,
    PublicationHistoryStatusDate,
    PlaceOfPublication,
    PubmedCentralIdentifier,
    PMCRelease,
    PubMedUniqueIdentifier,
    PersonalNameAsSubject,
    PublicationStatus,
    PublicationType,
    NumberOfReferences,
    RetractionIn,
    ECorRNNumber,
    RetractionOf,
    RepublishedFrom,
    RepublishedIn,
    RetractedAndRepublishedIn,
    RetractedAndPublishedFrom,
    Subset,
    SpaceFlightMission,
    SecondarySourceID,
    Source,
    SummaryForPatientsIn,
    StatusTag,
    JournalTitleAbbreviation,
    Title,
    TransliteratedTitle,
    UpdateIn,
    UpdateOf,
    Volume,
    VolumeTitle,
}

impl PubmedTag {
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "AB" => Some(Self::Abstract),
            "AD" => Some(Self::Affiliation),
            "AID" => Some(Self::ArticleIdentifier),
            "AU" => Some(Self::Author),
            "AUID" => Some(Self::AuthorIdentifier),
            "BTI" => Some(Self::BookTitle),
            // TODO everything else...
            _ => None,
        }
    }
}
