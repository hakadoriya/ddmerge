resource "aws_s3_bucket" "example" {
  # doc: https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/s3_bucket
  bucket = "my-${local.environment}-bucket"
}

resource "aws_s3_bucket_policy" "allow_access_from_another_account" {
  # doc: https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/s3_bucket_policy
  bucket = aws_s3_bucket.example.id
  policy = data.aws_iam_policy_document.allow_access_from_another_account.json
}

data "aws_iam_policy_document" "allow_access_from_another_account" {
  statement {
    principals {
      type        = "AWS"
      identifiers = ["123456789012"]
    }

    actions = [
      "s3:GetObject",
      "s3:ListBucket",
    ]

    resources = [
      aws_s3_bucket.example.arn,
      "${aws_s3_bucket.example.arn}/*",
    ]
  }
}
